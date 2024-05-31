use std::fs;
use std::io;
use anyhow::Ok;
use egui::RichText;
use serde_json::Value;
use eframe::egui;
use anyhow::Result;
use std::path::PathBuf;
use nfd::Response;
use native_dialog::{MessageDialog, MessageType};
struct Item { // Representation of a CDDA item
    data: Value,
}


fn get_property(data: &Value, property: &str) -> Option<String> { // Function to get a property from a JSON object
    data.get(property)?.as_str().map(|s| s.to_string())
}

fn get_name(item: &Item) -> Option<String> { // Function to get the name of an item
    // Access the 'name' field and its 'str' subfield
    if let Some(name) = item.data.get("name") { // Check if the 'name' field exists
        if let Some(name_str) = name.get("str") { // Check if the 'str' subfield exists
            if let Some(name_str) = name_str.as_str() { // Check if the 'str' subfield is a string
                return Some(name_str.to_string());
            }
        }
    }
    None // Return None if the name is not found
}


// TODO: Fix UI performance issues if possible
// TODO: Add a way to reselect the folder if the user wants to change it

fn main() -> Result<()>{
    // Show native prompt to select the game folder
    let _ = MessageDialog::new()
        .set_type(MessageType::Info)
        .set_title("CDDA Item Browser")
        .set_text("Please select your root CDDA folder (contains cataclysm-tiles.exe)")
        .show_alert()
        .unwrap();

    // Before app opens, show a file browser to select the game folder
    let result = nfd::open_pick_folder(None).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let selected_folder = match result { // Match the result of the file dialog
        Response::Okay(file_path) => { // If the user selected a file
            println!("File path = {:?}", file_path); // Print the file path
            file_path
        },
        Response::OkayMultiple(files) => { // If the user selected multiple files
            println!("Files {:?}", files);
            // Choose the first file from the multiple selection
            files.into_iter().next().unwrap_or_else(|| {
                println!("No file selected");
                "".to_string()
            })
        },
        Response::Cancel => { // If the user canceled the file dialog
            println!("User canceled");
            "".to_string()
        },
    };
    
    let game_folder = PathBuf::from(selected_folder).join("data/json/items");

     // Load all the json files in the json directory
    // let json_files: Result<Vec<PathBuf>, io::Error> = fs::read_dir(game_folder)?// fs::read_dir("./json")?
    //     .map(|res| res.map(|e| e.path()))
    //     .collect::<Result<Vec<PathBuf>, io::Error>>()
    //     .map_err(|err| err.into()); // Convert the error type
    // Load all the json files in all the subfolders of the json directory and the json directory itself
    let json_files: Result<Vec<PathBuf>, io::Error> = walkdir::WalkDir::new(game_folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.path().extension().map_or(false, |ext| ext == "json"))
        .map(|e| Ok(e.path().to_path_buf()))
        .collect::<Result<Vec<PathBuf>, anyhow::Error>>()
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string())); // Convert the error type

    // Vector to store the Item structs
    let mut items: Vec<Item> = Vec::new();

    // For each file, make an Item struct for each item in the file and put it into the vector
    for file in json_files?.iter().map(|file| file.as_ref()) { // Use the ? operator to propagate the error
        let file_contents = fs::read_to_string::<&std::path::Path>(file)?; // Specify the type annotation for the file variable
        let data: Vec<Value> = serde_json::from_str(&file_contents)?; // Parse the JSON data
        for item_data in data { // Iterate over the items inside the JSON file
            let item = Item { // Create a new Item struct
                data: item_data,
            };
            items.push(item);
        }
    }

    items.sort_by(|a, b| { // Sort the items by name
        let name_a = get_name(&a);
        let name_b = get_name(&b);
    
        match (name_a, name_b) { // Compare the names
            (Some(a), Some(b)) => a.cmp(&b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });
    
    // Remove all unnamed items
    items.retain(|item| get_name(&item).is_some());
    // Remove all "" named items
    items.retain(|item| get_name(&item).unwrap() != "");

    // Application options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 600.0]),
        ..Default::default()
    };
    
    // Application state variables
    let mut selected_item: Option<usize> = None;
    let mut search_text = String::new();

    // Run the application
    let _ = eframe::run_simple_native("Cataclysm: Dark Days Ahead Item Browser", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| { // Central panel
            ui.heading("Cataclysm: Dark Days Ahead Item Browser");
            // Search bar
            
            // Box to contain list of items
            ui.separator();
            ui.heading("Items");
            ui.horizontal(|ui| {
                ui.label("Search:");
                ui.text_edit_singleline(&mut search_text);
            });
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| { // Scroll area
                for (index, item) in items.iter().enumerate() {
                    if let Some(name) = item.data.get("name") { // Check if the 'name' field exists
                        if let Some(name_str) = name.get("str") { // Check if the 'str' subfield exists
                            if let Some(name_str) = name_str.as_str() { // Check if the 'str' subfield is a string
                                // Skip if the name is ""
                                if name_str != "" && name_str.to_lowercase().contains(search_text.to_lowercase().as_str()) {
                                    ui.horizontal(|ui| {
                                        if ui.selectable_value(&mut selected_item, Some(index), name_str).clicked() {
                                            selected_item = Some(index);
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
            });
        });
        // Item properties side panel
        egui::SidePanel::right("side_panel").default_width(300.0).show(ctx, |ui| {
            ui.heading("Information");
            ui.separator();
            // TODO: Clean this part up, it's redundant and messy in some places
            if let Some(index) = selected_item {
                // Display the information of the selected item
                ui.heading(format!("{}", get_name(&items[index]).unwrap()));
                // Description
                if let Some(description) = get_property(&items[index].data, "description") {
                    ui.label(format!("{}", description));
                }
                ui.separator();
                // Volume
                if let Some(volume) = get_property(&items[index].data, "volume") {
                    ui.label(RichText::new("Volume:").strong());
                    ui.label(format!("{}", volume));
                }
                // Weight
                if let Some(weight) = get_property(&items[index].data, "weight") {
                    ui.label(RichText::new("Weight:").strong());
                    ui.label(format!("{}", weight));
                }
                // Price
                if let Some(price) = get_property(&items[index].data, "price_postapoc") {
                    ui.label(RichText::new("Price:").strong());
                    ui.label(format!("{}", price));
                }
                // Material
                if let Some(material) = get_property(&items[index].data, "material") {
                    ui.label(RichText::new("Material:").strong());
                    ui.label(format!("{}", material));
                }
                // Flags
                if let Some(flags) = items[index].data.get("flags") {
                    ui.label(RichText::new("Flags:").strong());
                    for flag in flags.as_array().unwrap() {
                        ui.label(format!("\t{}", flag.as_str().unwrap()));
                    }
                }
                ui.separator();
                // Display the rest of the properties
                for (key, value) in items[index].data.as_object().unwrap() {
                    // If the line starts with a //, skip over it (developer comments)
                    if key.starts_with("//") {
                        continue;
                    }
                    let key = {
                        let mut chars = key.chars();
                        chars.next().unwrap().to_uppercase().collect::<String>() + chars.as_str()
                    };
                    if key != "Name" && key != "Description" && key != "Volume" && key != "Weight" && key != "Price_postapoc" && key != "Price" && key != "Material" && key != "Flags" {
                        // If a property is a list of values, list them out nicely
                        if value.is_array() {
                            ui.label(RichText::new(format!("{}:", key)).strong());
                            for item in value.as_array().unwrap() {
                                if item.is_array() {
                                    for subitem in item.as_array().unwrap() {
                                        // If it's a number, don't make a new label
                                        if subitem.is_number() {
                                            // ui.label(format!("\t {}", subitem));
                                        } else if subitem.is_string() {
                                            ui.label(format!("\t {}", subitem.as_str().unwrap()));
                                        }
                                    }
                                } else if item.is_string() {
                                    ui.label(format!("\t  {}", item.as_str().unwrap()));
                                }
                            }
                        } else if value.is_string() {
                            ui.label(RichText::new(format!("{}:", key)).strong());
                            ui.label(format!("{}", value.as_str().unwrap()));
                        }
                    }
                }
            } else {
                ui.label("No item selected");
            }
        });
    });
    Ok(())
}