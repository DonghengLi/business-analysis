use std::collections::{HashMap, HashSet};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "encounterId")]
    encounter_id: String,
    doctor: String,
    division: String,
    #[serde(rename = "processedSOAP")]
    processed_soap: String,
}

#[derive(PartialEq, Eq, Hash, Debug)]
struct Key {
    doctor: String,
    division: String,
}

const WHITE_SPACE: u32 = 1;
const WORD: u32 = 2;
const OTHER: u32 = 4;

#[derive(Debug, Deserialize)]
struct SubjectInfo {
    #[serde(rename = "SUBJECT_ID")]
    subject_id: String,
    #[serde(rename = "SUBJECT_NAME")]
    subject_name: String,
}

fn load_subject() -> anyhow::Result<HashMap<String, String>> {
    let mut reader = csv::Reader::from_path("subject.csv")?;
    let mut map = HashMap::new();
    for record in reader.deserialize() {
        let SubjectInfo { subject_id, subject_name } = record?;
        map.insert(subject_id, subject_name);
    }
    Ok(map)
}

const MIN_WORDS_NUM: usize = 4;
const MAX_WORDS_NUM: usize = 10;

fn get_output() -> anyhow::Result<()> {
    let subject_map = load_subject()?;

    let mut reader = csv::Reader::from_path("sample.csv")?;
    let mut buffer = String::new();
    let mut data: HashMap<Key, HashMap<String, Vec<String>>> = HashMap::new();

    // TODO: remove this at root.
    let mut encounter_ids: HashSet<String> = HashSet::new();
    
    for record in reader.deserialize() {
        let Record { encounter_id, doctor, division, processed_soap  } = record?;
        if encounter_ids.contains(&encounter_id) {
            continue;
        }
        encounter_ids.insert(encounter_id.clone());

        let soap: Vec<Vec<(String, u32)>> = serde_json::from_str(&processed_soap)?;
        let key = Key {
            doctor, division,
        };
        
        let sentences = data.entry(key).or_insert_with(HashMap::new);
        
        for words in &soap {
            for i in 0..words.len() {
                if words[i].1 != WORD {
                    continue;
                }
                buffer.clear();
                let mut words_num: usize = 0;
                for j in i..words.len() {
                    buffer.push_str(&words[j].0);
                    
                    if words[j].1 != WORD {
                        continue;
                    }
                    
                    words_num += 1;
                    if words_num > MAX_WORDS_NUM {
                        break;
                    }
                    if words_num >= MIN_WORDS_NUM {
                        let encounter_ids = sentences.entry(buffer.clone()).or_insert_with(Vec::new);
                        encounter_ids.push(encounter_id.clone());
                    }
                }
            }
        }
    }

    let mut writer = csv::Writer::from_path("output.csv")?;
    writer.write_record(&["doctor", "division", "sentence", "frequency", "encounter_ids"])?;

    for (key, value) in data {
        for (sentence, encounter_ids) in value {
            let frequency = encounter_ids.len();
            if frequency == 1 {
                continue;
            }
            let division = subject_map.get(&key.division).unwrap_or(&key.division);
            let encounter_ids: String = encounter_ids.join(", ");
            writer.write_record(&[&key.doctor, division, &sentence, &frequency.to_string(), &encounter_ids])?;
        }
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    get_output()?;
    seq_words::process_output()
}
