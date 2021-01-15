use yew::prelude::*;
use yew::virtual_dom::VNode;

use crate::file::ScratchFile;
use crate::fileinput::FileInput;
use crate::sprite::SpriteID;
use crate::thread::BlockInputs;
use crate::vm::VM;

use super::*;

pub struct FileViewer {
    link: ComponentLink<Self>,
    block_inputs: HashMap<SpriteID, BlockInputsName>,
    file_text: String,
}

pub enum Msg {
    LoadFile(ScratchFile),
    SetBlockInputs(HashMap<SpriteID, BlockInputsName>),
}

impl FileViewer {
    fn sprite(block_inputs: &HashMap<SpriteID, BlockInputsName>) -> Vec<VNode> {
        block_inputs
            .iter()
            .map(|(sprite_id, block_inputs)| {
                let sprite_id_truncated: String =
                    format!("{}", sprite_id).chars().take(8).collect();
                html! {
                    <>
                        <h1>
                            <strong>
                                {format!("Sprite {} ({})", sprite_id_truncated, block_inputs.name)}
                            </strong>
                        </h1>
                        {FileViewer::thread(&block_inputs.block_inputs)}
                    </>
                }
            })
            .collect()
    }

    fn thread(thread_blocks: &[BlockInputs]) -> Vec<VNode> {
        thread_blocks
            .iter()
            .enumerate()
            .map(|(thread_id, block_inputs)| {
                html! {
                    <>
                        <h2><strong>{format!("Thread {}", thread_id)}</strong></h2>
                        <Diagram block_inputs={block_inputs} />
                    </>
                }
            })
            .collect()
    }
}

impl Component for FileViewer {
    type Message = Msg;
    type Properties = ();

    fn create(_: (), link: ComponentLink<Self>) -> Self {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .set_title("Scratch File Viewer");

        Self {
            link,
            block_inputs: HashMap::new(),
            file_text: String::new(),
        }
    }

    fn update(&mut self, msg: Msg) -> bool {
        match msg {
            Msg::LoadFile(file) => {
                self.file_text = format!("{:#?}", &file.project);
                let set_block_inputs = self.link.callback(Msg::SetBlockInputs);
                wasm_bindgen_futures::spawn_local(async move {
                    match VM::block_inputs(&file).await {
                        Ok(mut block_input) => {
                            let mut id_name: HashMap<SpriteID, String> = file
                                .project
                                .targets
                                .iter()
                                .map(|t| (SpriteID::from_sprite_name(&t.name), t.name.clone()))
                                .collect();
                            let block_inputs: HashMap<SpriteID, BlockInputsName> = block_input
                                .iter_mut()
                                .map(|(id, block_inputs)| {
                                    (
                                        *id,
                                        BlockInputsName {
                                            name: id_name.remove(id).unwrap(),
                                            block_inputs: std::mem::take(block_inputs),
                                        },
                                    )
                                })
                                .collect();
                            set_block_inputs.emit(block_inputs);
                        }
                        Err(e) => {
                            log::error!("{}", e);
                        }
                    };
                });

                false
            }
            Msg::SetBlockInputs(block_inputs) => {
                self.block_inputs = block_inputs;
                true
            }
        }
    }

    fn change(&mut self, _: ()) -> bool {
        unreachable!()
    }

    fn view(&self) -> Html {
        html! {
            <>
                <FileInput onchange={self.link.callback(Msg::LoadFile)} />
                <style>
                    {"br { margin-bottom: 2px; }"}
                </style>
                <span style="font-family: monospace;">
                    {FileViewer::sprite(&self.block_inputs)}
                </span>

                {
                    if self.file_text.len() > 0 {
                        html! {
                            <>
                                <br />
                                <h1 style="font-family: monospace;">{"ScratchFile structure"}</h1>
                                <pre>
                                    {self.file_text.clone()}
                                </pre>
                            </>
                        }
                    } else {
                        html! {}
                    }
                }
            </>
        }
    }
}

#[derive(Debug)]
pub struct BlockInputsName {
    name: String,
    block_inputs: Vec<BlockInputs>,
}

#[derive(Debug)]
struct Diagram {
    block_inputs: RefCell<BlockInputs>,
}

#[derive(Clone, Properties, PartialEq)]
struct DiagramProps {
    block_inputs: BlockInputs,
}

impl Component for Diagram {
    type Message = ();
    type Properties = DiagramProps;

    fn create(props: DiagramProps, _: ComponentLink<Self>) -> Self {
        Self {
            block_inputs: RefCell::new(props.block_inputs),
        }
    }

    fn update(&mut self, _: ()) -> bool {
        false
    }

    fn change(&mut self, props: DiagramProps) -> bool {
        if *self.block_inputs.borrow() != props.block_inputs {
            self.block_inputs = RefCell::new(props.block_inputs);
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let mut block_inputs = self.block_inputs.borrow_mut();
        let next_html = if let Some(next) = block_inputs.stacks.remove("next") {
            html! {<><Diagram block_inputs={next} /></>}
        } else {
            html! {}
        };

        let mut substacks: Vec<(&'static str, BlockInputs)> = block_inputs.stacks.drain().collect();
        substacks.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

        html! {
            <>
                <strong>{block_inputs.info.name.to_string()}</strong>
                {String::from(" ") + &block_inputs.info.id.to_string()}<br />

                <div style="padding-left: 10px; border-left: 1px solid #B3B3B3; margin-bottom: 1px;">
                    {
                        for block_inputs.fields.iter().map(|field| {
                            html! {
                                <>
                                    {field.0.to_string() + ": " + &field.1}
                                    <br />
                                </>
                            }
                        })
                    }
                    {
                        for block_inputs.inputs.drain().map(|input| {
                            html! {
                                <>
                                    {input.0.to_string() + ": "}
                                    <Diagram block_inputs={input.1} />
                                </>
                            }
                        })
                    }
                    {
                        for substacks.drain(..).map(|substack| {
                            html! {
                                <>
                                    {substack.0}<br />
                                    <Diagram block_inputs={substack.1} />
                                </>
                            }
                        })
                    }
                </div>
                {next_html}
            </>
        }
    }
}