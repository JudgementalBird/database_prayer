use std::sync::Arc;
use colored::{Color, Colorize};
use dialoguer::Input;
use tokio::sync::Mutex;
use rusqlite::{Connection, Error};

pub struct SLDatabase {
	pub connection: Arc<Mutex<Connection>>
}
impl SLDatabase {
	pub async fn ensure_withdrawals_table_exists(&self) {
		let locked_fish_database = self.connection.lock().await;
		locked_fish_database.execute(
			"CREATE TABLE IF NOT EXISTS withdrawals (
			crane_id INTEGER NOT NULL,
			steam_id BIGINT NOT NULL,
			specific_withdrawn BLOB NOT NULL,
			received_at BIGINT NOT NULL
			)",()
		).expect("Failed at creating a table if (one doesn't exist)..");
	}
	pub async fn setup() -> Result<Self,Error> {
		let database = Self {
			connection: Arc::new(Mutex::new(
				Connection::open("./fish_addon_db.db3")?
				))
			};
		
		database.ensure_withdrawals_table_exists().await;

		Ok(database)
	}
}

pub fn name_from_array_index(species: usize) -> Result<String, String> {
	const SPECIES_NAMES: &[&str] = &["Anchovie","Anglerfish","Arctic Char","Ballan Lizardfish","Ballan Wrasse","Barreleye Fish","Black Bream","Black Dragonfish","Clown Fish","Cod","Dolphinfish","Gulper Eel","Haddock","Hake","Herring","John Dory","Labrus","Lanternfish","Mackerel","Midshipman","Perch","Pike","Pinecone Fish","Pollock","Red Mullet","Rockfish","Sablefish","Salmon","Sardine","Scad","Sea Bream","Sea Halibut","Sea Piranha","Seabass","Slimehead","Snapper","Snapper (Gold)","Snook","Spadefish","Trout","Tubeshoulders Fish","Viperfish","Yellowfin Tuna","Blue Crab","Brown Box Crab","Coconut Crab","Dungeness Crab","Furry Lobster","Homarus Americanus","Homarus Gammarus","Horseshoe Crab","Jasus Edwardsii","Jasus Lalandii","Jonah Crab","King Crab","Mud Crab","Munida Lobster","Ornate Rock Lobster","Panulirus Interruptus","Red King Crab","Reef Lobster","Slipper Lobster","Snow Crab","Southern Rock Lobster","Spider Crab","Spiny Lobster","Stone Crab"];

	SPECIES_NAMES.get(species)
		.map(|name| name.to_string())
		.ok_or_else(|| String::from("Error! Sea creature {}"))
}
pub fn to_display_string(data: &Vec<u32>) -> Result<String, String> {
	let mut result = String::new();
	
	for (species, quantity) in data.iter().enumerate() {
		if *quantity > 0 {
			if !result.is_empty() {
				result.push_str(", ");
			}
			result.push_str(&quantity.to_string());
			result.push(' '); 
			match name_from_array_index(species) {
				Ok(name) => result.push_str(&name),
				Err(e) => result.push_str(&e),
			}
		}
	}
	Ok(result)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("{}","A prayer for the database has been opened !".magenta());
    let db = SLDatabase::setup().await.expect("Failed to open DB!");
    println!("DB opened successfully");
    loop {
        let connection = db.connection.lock().await;
        
        let query_timestamp = Input::<u64>::new()
            .with_prompt("What unix timestamp should be queried?")
            .interact_text()
            .unwrap();

        println!("Querying {} !",query_timestamp);

        let res = connection.query_row(
            "SELECT * FROM withdrawals WHERE received_at = ?",
            [query_timestamp],
            |row| {
                let fish_data: Vec<u8> = row.get(2).unwrap();
                let fish_data: Vec<u32> = bincode::deserialize(&fish_data).unwrap();

                println!(
                    "{} {} {}",
                    "-".repeat(40).magenta(),
                    "Success".magenta(),
                    "-".repeat(100).magenta());
                
                let (data_string, index_string) = fish_data.iter()
                    .enumerate()
                    .map(|(species, &quantity)| {
                        let to_use = match quantity {
                            0 => Color::Yellow,
                            _ => Color::Green
                        };
                        let data = if species >= 10 && quantity < 10 {
                            format!("{}, ",
                                format!("0{}", quantity).color(to_use)
                            )
                        } else {
                            format!("{}, ",quantity.to_string().color(to_use))
                        };

                        let index = format!("{}, ",species.to_string().yellow());

                        (data,index)
                    })
                    .fold((String::new(), String::new()), |(mut data_acc, mut index_acc), (data, index)| {
                        data_acc.push_str(&data);
                        index_acc.push_str(&index);
                        (data_acc, index_acc)
                    });
                
                    println!(" Indexes: {}", index_string);
                    println!("Vec<u32>: {}", data_string);
                    println!("   Names: {}", to_display_string(&fish_data).unwrap().green());
                Ok(())
            });
        
        match res {
            Ok(_) => {},
            Err(e) => println!("rusqlite error: {:?}", e),
        };
    }
}