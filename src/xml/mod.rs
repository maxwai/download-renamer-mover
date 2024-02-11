use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;

use log::{error, info, warn};
use xmltree::XMLNode::Text;
use xmltree::{Element, XMLNode};

const DUMMY_CONTENT: &str = r##"
<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<root>
  <BotToken><!--Put here your Bot Token--></BotToken>
  <MainChannel><!--Put here the Channel ID of the Main Channel--></MainChannel>
</root>"##;

const CONFIG_FILE_NAME: &str = "appdata/Config.xml";

const BOT_TOKEN_TAG: &str = "BotToken";
const MAIN_CHANNEL_TAG: &str = "MainChannel";

// Mappings
const MAPPINGS_TAG: &str = "Mappings";
const MAPPING_SINGLE_TAG: &str = "Mapping";
const ALTERNATIVE_ATTRIBUTE_TAG: &str = "alternative";
// Mappings

/// Saves a dummy document and then exits
fn save_dummy_document() {
    let dummy_element: Element = Element::parse(DUMMY_CONTENT.as_bytes()).unwrap();
    match dummy_element.write(File::create(CONFIG_FILE_NAME).unwrap()) {
        Ok(_) => info!("Created dummy file."),
        Err(error) => error!("{error}"),
    }
    error!("There was no {CONFIG_FILE_NAME} available. Created a dummy one. Please fill it out");
    exit(1);
}

/// Will write the new XML file to Config.xml
fn write_document(document: Element) {
    match document.write(File::create(CONFIG_FILE_NAME).unwrap()) {
        Ok(_) => info!("Saved the Config.xml"),
        Err(error) => error!("Could not save correctly the XML File.\n{error}"),
    }
}

/// Will get the Config.xml or, if not present, create a dummy one and exit
fn get_document() -> Element {
    let file_path = Path::new(CONFIG_FILE_NAME);
    match file_path.try_exists() {
        Ok(status) => {
            if !status {
                save_dummy_document();
            }
        }
        Err(_) => save_dummy_document(),
    }
    let file = File::open(file_path).unwrap();
    match Element::parse(file) {
        Ok(element) => element,
        Err(error) => {
            error!("Something went wrong while parsing the xml: {error}");
            exit(1);
        }
    }
}

/// Will retrieve the Discord Bot Token
pub fn get_bot_token() -> String {
    let document = get_document();
    match document.get_child(BOT_TOKEN_TAG) {
        None => {
            error!("No Bot Token found");
            panic!();
        }
        Some(element) => match element.get_text() {
            None => {
                error!("No Bot Token found");
                panic!();
            }
            Some(token) => {
                info!("Getting the Bot Token");
                token.to_string()
            }
        },
    }
}

/// Will retrieve the Main Channel ID
pub fn get_main_channel() -> u64 {
    let document = get_document();
    match document.get_child(MAIN_CHANNEL_TAG) {
        None => {
            error!("No Main Channel ID found");
            panic!();
        }
        Some(element) => match element.get_text() {
            None => {
                error!("No Main Channel ID found");
                panic!();
            }
            Some(token) => {
                info!("Getting the Main Channel ID");
                return match u64::from_str(token.as_ref()) {
                    Ok(value) => value,
                    Err(_) => {
                        error!("Main Channel ID is not an u64");
                        panic!();
                    }
                };
            }
        },
    }
}

/// Will get known Mappings if there are any
///
/// The Entries in the HashMap are like this: (alt -> OG)
pub fn get_mappings() -> HashMap<String, String> {
    let document = get_document();
    match document.get_child(MAPPINGS_TAG) {
        None => {
            info!("No Mappings known");
            HashMap::new()
        }
        Some(element) => {
            let mut output = HashMap::new();
            element.children.iter().for_each(|child| {
                if let XMLNode::Element(element) = child {
                    if element.name == MAPPING_SINGLE_TAG {
                        match element.attributes.get(ALTERNATIVE_ATTRIBUTE_TAG) {
                            None => warn!("Got Mapping without {ALTERNATIVE_ATTRIBUTE_TAG} Tag"),
                            Some(attribute) => match element.get_text() {
                                None => warn!("Got Mapping without Text"),
                                Some(og_text) => {
                                    output.insert(attribute.to_string(), og_text.to_string());
                                }
                            },
                        }
                    } else {
                        warn!("Got unknown Tag: {}", element.name);
                    }
                }
            });
            output
        }
    }
}

/// Will add a Mapping to the Mappings
pub fn add_mappings<S, U>(old: S, og: U)
where
    S: Into<String>,
    U: Into<String>,
{
    let mut document = get_document();
    let mut temp: Element;
    let mappings: &mut Element = match document.get_mut_child(MAPPINGS_TAG) {
        None => {
            temp = Element::new(MAPPINGS_TAG);
            &mut temp
        }
        Some(element) => element,
    };
    let text = Text(og.into());

    let mut mapping = Element::new(MAPPING_SINGLE_TAG);
    mapping
        .attributes
        .insert(ALTERNATIVE_ATTRIBUTE_TAG.to_string(), old.into());
    mapping.children = vec![text];

    let mut children = mappings.children.to_vec();
    children.push(XMLNode::Element(mapping));
    mappings.children = children;

    info!("Added a Mapping");
    write_document(document);
}
