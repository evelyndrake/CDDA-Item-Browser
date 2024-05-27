use std::fs;
use std::io;
use serde_json::Value;
use colored::Colorize;

struct Item { // Representation of a CDDA item
    data: Value,
    number: i32,
}

fn get_property(data: &Value, property: &str) -> Option<String> { // Function to get a property from a JSON object
    data.get(property)?.as_str().map(|s| s.to_string())
}

 
fn main() -> io::Result<()> {
    // Load all the json files in the json directory
    let json_files = fs::read_dir("./json")?
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;

    // Vector to store the Item structs
    let mut items: Vec<Item> = Vec::new();
    let mut index = 0;

    // For each file, make an Item struct for each item in the file and put it into the vector
    for file in json_files { // Iterate over the files
        let file_contents = fs::read_to_string(file.clone())?; // Read the file contents
        let data: Vec<Value> = serde_json::from_str(&file_contents)?; // Parse the JSON data
        for item_data in data { // Iterate over the items inside the JSON file
            let item = Item { // Create a new Item struct
                data: item_data,
                number: index,
            };
            index += 1;
            items.push(item);
        }
    }

    // Display information
    println!("-------------------");
    println!("{}","CATACLYSM DDA ITEM BROWSER".green().bold());
    println!("There are {} items available.", items.len());
    // Wait for user to hit enter
    println!("Press enter to continue...");
    let _ = std::io::stdin().read_line(&mut String::new());
    println!("-------------------");

    loop { // Main loop
        for item in &items { // Iterate over the items
            // Access the 'name' field and its 'str' subfield
            if let Some(name) = item.data.get("name") { // Check if the 'name' field exists
                if let Some(name_str) = name.get("str") { // Check if the 'str' subfield exists
                    if let Some(name_str) = name_str.as_str() { // Check if the 'str' subfield is a string
                        println!("{}.) {}", item.number, name_str); // Print the item number and name
                    }
                }
            }
        }
    
        // Prompt the user to select an item
        println!("Select an item (or type 'exit' to quit):");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        if input.trim() == "exit" {
            break;
        }
        let selection: usize = input.trim().parse().unwrap();
        // Get the selected item
        let selected_item = &items[selection];
        println!("-------------------");
        // Print the selected item's name
        if let Some(name) = selected_item.data.get("name") {
            if let Some(name_str) = name.get("str") {
                if let Some(name_str) = name_str.as_str() {
                    println!("{}", name_str.green().bold());
                }
            }
        }
        println!("-------------------");
        // Print the selected item's description
        if let Some(description) = get_property(&selected_item.data, "description") {
            println!("{}", description.yellow());
        }
        // Print the rest of the properties, list the name and the value for each
        for (key, value) in selected_item.data.as_object().unwrap() {
            if key != "name" && key != "description" {
                println!("{}: {}", key.italic(), value);
            }
        }

        // Wait for user to hit enter
        println!("Press enter to continue...");
        let _ = std::io::stdin().read_line(&mut String::new());
        println!("-------------------"); // Print a horizontal line at the end
    }
    Ok(()) // Return an Ok result
}