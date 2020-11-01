use super::*;
use blocks::BlockInputs;
use futures::future::LocalBoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt};
use gloo_timers::future::TimeoutFuture;
use runtime::{Global, Runtime};
use savefile::ScratchFile;
use sprite::Sprite;
use tokio::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug)]
pub struct VM {
    control_chan: Sender<Control>,
    block_inputs: Vec<Vec<BlockInputs>>,
}

#[derive(Debug, Copy, Clone)]
enum Control {
    Continue,
    Pause,
    Step,
}

impl VM {
    pub async fn start(
        context: web_sys::CanvasRenderingContext2d,
        scratch_file: &ScratchFile,
    ) -> Result<Self> {
        let global = Global::new(&scratch_file.project.targets[0].variables);

        let mut sprites: Vec<Sprite> = Vec::with_capacity(scratch_file.project.targets.len());
        for target in &scratch_file.project.targets {
            let runtime =
                runtime::SpriteRuntime::new(&target.costumes, &scratch_file.images).await?;

            let runtime = Runtime {
                sprite: Rc::new(RwLock::new(runtime)),
                global: global.clone(),
            };

            sprites.push(Sprite::new(runtime, target).await?);
        }

        let block_inputs: Vec<Vec<BlockInputs>> =
            sprites.iter().map(|s| s.block_inputs()).collect();

        let (sender, receiver) = channel(1);
        wasm_bindgen_futures::spawn_local(async move {
            match VM::run(sprites, receiver, &context).await {
                Ok(_) => {}
                Err(e) => log::error!("{}", e),
            }
        });

        Ok(Self {
            control_chan: sender,
            block_inputs,
        })
    }

    async fn redraw(sprites: &[Sprite], context: &web_sys::CanvasRenderingContext2d) -> Result<()> {
        let mut need_redraw = false;
        for sprite in sprites.iter() {
            if sprite.need_redraw().await {
                need_redraw = true;
                break;
            }
        }

        if !need_redraw {
            return Ok(());
        }

        context.reset_transform().unwrap();
        context.scale(2.0, 2.0).unwrap();

        for (i, sprite) in sprites.iter().enumerate() {
            sprite.redraw(&context, i == 0).await?;
        }
        Ok(())
    }

    async fn run(
        sprites: Vec<Sprite>,
        control_chan: Receiver<Control>,
        context: &web_sys::CanvasRenderingContext2d,
    ) -> Result<()> {
        const REDRAW_INTERVAL_MILLIS: u32 = 17;

        let control_chan = ReceiverCell::new(control_chan);
        let mut futures: FuturesUnordered<LocalBoxFuture<Event>> = FuturesUnordered::new();
        futures.push(Box::pin(control_chan.recv()));
        futures.push(Box::pin(
            TimeoutFuture::new(REDRAW_INTERVAL_MILLIS).map(|_| Event::Redraw),
        ));

        for (sprite_id, sprite) in sprites.iter().enumerate() {
            for thread_id in 0..sprite.number_of_threads() {
                let id = ThreadID {
                    sprite_id,
                    thread_id,
                };
                futures.push(VM::step_sprite(&sprites[id.sprite_id], id))
            }
        }

        let mut current_state = Control::Continue;
        let mut paused_threads: Vec<ThreadID> = Vec::new();

        loop {
            match futures.next().await.unwrap() {
                Event::Thread(thread_id) => {
                    match current_state {
                        Control::Continue => {
                            futures.push(VM::step_sprite(&sprites[thread_id.sprite_id], thread_id))
                        }
                        Control::Pause => paused_threads.push(thread_id),
                        Control::Step => {
                            futures.push(VM::step_sprite(&sprites[thread_id.sprite_id], thread_id));
                            current_state = Control::Pause;
                        }
                    }
                    // TODO find a way to yield to redraw
                    // Check event stack to see if redraw is being added
                    TimeoutFuture::new(0).await;
                }
                Event::Error(e) => return Err(e),
                Event::Redraw => {
                    VM::redraw(&sprites, context).await?;
                    futures.push(Box::pin(
                        TimeoutFuture::new(REDRAW_INTERVAL_MILLIS).map(|_| Event::Redraw),
                    ));
                }
                Event::Control(control) => {
                    if let Some(c) = control {
                        current_state = c;
                        match c {
                            Control::Continue | Control::Step => {
                                for thread_id in paused_threads.drain(..) {
                                    futures.push(VM::step_sprite(
                                        &sprites[thread_id.sprite_id],
                                        thread_id,
                                    ))
                                }
                            }
                            Control::Pause => {}
                        }
                        log::info!("{:?}", c);
                    }
                    futures.push(Box::pin(control_chan.recv()));
                }
            };
        }
    }

    fn step_sprite(sprite: &Sprite, thread_id: ThreadID) -> LocalBoxFuture<Event> {
        Box::pin(
            sprite
                .step(thread_id.thread_id)
                .map(move |result| match result {
                    Ok(_) => Event::Thread(thread_id),
                    Err(e) => Event::Error(e),
                }),
        )
    }

    pub async fn continue_(&mut self) {
        self.control_chan.send(Control::Continue).await.unwrap();
    }

    pub async fn pause(&mut self) {
        self.control_chan.send(Control::Pause).await.unwrap();
    }

    pub async fn step(&mut self) {
        self.control_chan.send(Control::Step).await.unwrap();
    }

    pub fn block_inputs(&self) -> Vec<Vec<BlockInputs>> {
        self.block_inputs.clone()
    }
}

#[derive(Debug)]
enum Event {
    Thread(ThreadID),
    Error(Error),
    Redraw,
    Control(Option<Control>),
}

#[derive(Debug, Copy, Clone)]
struct ThreadID {
    sprite_id: usize,
    thread_id: usize,
}

/// Resolves a lifetime issue with Receiver and FuturesUnordered.
#[derive(Debug)]
struct ReceiverCell {
    receiver: RefCell<Receiver<Control>>,
}

impl ReceiverCell {
    fn new(receiver: Receiver<Control>) -> Self {
        Self {
            receiver: RefCell::new(receiver),
        }
    }

    async fn recv(&self) -> Event {
        Event::Control(self.receiver.borrow_mut().recv().await)
    }
}
