#![allow(clippy::needless_range_loop)]

use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    rc::Rc, io::Write,
};

/// Input CSV record.
#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "encounterId")]
    encounter_id: String,
    doctor: String,
    division: String,
    #[serde(rename = "processedSOAP")]
    processed_soap: String,
}

/// Key to group the records.
#[derive(PartialEq, Eq, Hash, Debug)]
struct Key {
    doctor: String,
    division: String,
}

/// The values for each key.
struct Value {
    encounter_id: Rc<str>,
    soap: Vec<Vec<(String, u32)>>,
}

// Define aligned with the output data of [../soap-sep]
// const WHITE_SPACE: u32 = 1;
const WORD: u32 = 2;
// const OTHER: u32 = 4;

// Subject CSV record.
#[derive(Debug, Deserialize)]
struct SubjectInfo {
    #[serde(rename = "SUBJECT_ID")]
    subject_id: String,
    #[serde(rename = "SUBJECT_NAME")]
    subject_name: String,
}

/// Load the subject maps from the CSV file.
fn load_subject(path: &Path) -> anyhow::Result<HashMap<String, String>> {
    let mut reader = csv::Reader::from_path(path)?;
    let mut map = HashMap::new();
    for record in reader.deserialize() {
        let SubjectInfo {
            subject_id,
            subject_name,
        } = record?;
        map.insert(subject_id, subject_name);
    }
    Ok(map)
}

fn main() -> anyhow::Result<()> {
    let matches = clap::Command::new("seq-words")
        .version("0.1.0")
        .author("Dongheng Li")
        .about("Finds the repeated sentences in the SOAP notes.")
        .arg(
            clap::Arg::new("min_words")
                .short('m')
                .long("min-words")
                .value_name("MIN_WORDS")
                .help("Sets the minimum number of words in a sentence")
                .value_parser(clap::value_parser!(usize))
                .required(true),
        )
        .arg(
            clap::Arg::new("input")
                .short('i')
                .long("input")
                .value_name("INPUT_FILE")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Sets the input CSV file path")
                .required(true),
        )
        .arg(
            clap::Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT_FILE")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Sets the output CSV file path")
                .required(true),
        )
        .arg(
            clap::Arg::new("subject")
                .short('s')
                .long("subject")
                .value_name("SUBJECT_FILE")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Sets the subject mapping CSV file path")
                .required(true),
        )
        .get_matches();

    let min_words: usize = *matches.get_one("min_words").unwrap();
    let input_file: &Path = matches.get_one::<PathBuf>("input").unwrap();
    let output_file: &Path = matches.get_one::<PathBuf>("output").unwrap();
    let subject_file: &Path = matches.get_one::<PathBuf>("subject").unwrap();

    let subject_map = load_subject(subject_file)?;

    // Loads the data and group them by doctor and division.
    let mut records = HashMap::new();
    for record in csv::Reader::from_path(input_file)?.deserialize() {
        let Record {
            encounter_id,
            doctor,
            division,
            processed_soap,
        } = record?;
        let encounter_id: Rc<str> = encounter_id.into();
        let key = Key { doctor, division };
        let value = Value {
            encounter_id,
            soap: serde_json::from_str(&processed_soap)?,
        };
        records.entry(key).or_insert_with(Vec::new).push(value);
    }

    let mut writer = std::fs::File::create(output_file)?;
    // Write UTF-8 BOM to make Excel happy.
    writer.write_all(&[0xEF, 0xBB, 0xBF])?;
    let mut writer = csv::Writer::from_writer(writer);
    writer.write_record([
        "doctor",
        "division",
        "sentence",
        "frequency",
        "encounter_ids",
    ])?;
    // Process each doctor and division.
    for (key, value) in records {
        let repeated_min_words = get_repeated_min_words(&value, min_words);
        let all_repeated_sentences =
            get_all_repeated_sentences(&repeated_min_words, &value, min_words);
        let repeated_sentences = remove_subset(all_repeated_sentences);

        for (sentence, encounter_ids) in repeated_sentences {
            let frequency = encounter_ids.len();
            let division = subject_map.get(&key.division).unwrap_or(&key.division);
            let encounter_ids: String = encounter_ids.join(", ");
            writer.write_record([
                &key.doctor,
                division,
                &sentence,
                &frequency.to_string(),
                &encounter_ids,
            ])?;
        }
    }
    Ok(())
}

/// Get all the minimal number of words sentences that starts with a word and ends with a word.
fn get_repeated_min_words(values: &[Value], min_words_num: usize) -> HashSet<String> {
    let mut word_frequency = HashMap::<String, usize>::new();
    for value in values {
        for soap in &value.soap {
            let mut buffer = seq_words::SentenceRingBuffer::new(min_words_num);
            for (word, word_type) in soap {
                if let Some(sentence) = buffer.add(word.clone(), *word_type == WORD) {
                    *word_frequency.entry(sentence).or_insert(0) += 1;
                }
            }
        }
    }
    // Create a set of sentences that have a frequency greater than 1.
    word_frequency
        .into_iter()
        .filter(|(_, frequency)| *frequency > 1)
        .map(|(sentence, _)| sentence)
        .collect()
}

/// A maps of all the repeated sentences and the occurance of encounter IDs.
/// If the sentence is repeated more than once in the same encounter, the encounter ID is added multiple times.
fn get_all_repeated_sentences(
    repeated_min_words: &HashSet<String>,
    values: &[Value],
    min_words_num: usize,
) -> HashMap<String, Vec<Rc<str>>> {
    let mut sentence_encounters = HashMap::new();
    for value in values {
        for soap in &value.soap {
            for i in 0..soap.len() {
                let mut full_sentence = String::new();
                let mut buffer = seq_words::SentenceRingBuffer::new(min_words_num);

                for j in i..soap.len() {
                    let (word, word_type) = &soap[j];
                    full_sentence.push_str(word);
                    if let Some(sentence) = buffer.add(word.clone(), *word_type == WORD) {
                        // There are enough words in the sentence when the buffer returns a sentence.
                        if !repeated_min_words.contains(&sentence) {
                            // If the sub-sentence between the last minimal number of words does not occur multiple time,
                            // then we don't need to check the rest of the SOAP note.
                            break;
                        }
                        sentence_encounters
                            .entry(full_sentence.clone())
                            .or_insert_with(Vec::new)
                            .push(value.encounter_id.clone());
                    }
                }
            }
        }
    }
    sentence_encounters
        .into_iter()
        .filter(|(_, encounter_ids)| encounter_ids.len() > 1)
        .collect()
}

/// Remove the subset sentence with no more frequency.
fn remove_subset(input: HashMap<String, Vec<Rc<str>>>) -> Vec<(String, Vec<Rc<str>>)> {
    let mut result: Vec<(String, Vec<Rc<str>>)> = Vec::new();
    let mut input = input.into_iter().collect::<Vec<_>>();
    // Sort the sentences by the lengh, so the possible subset of a sentence is always before the sentence.
    input.sort_by(|(sentence1, _), (sentence2, _)| sentence1.len().cmp(&sentence2.len()));
    for i in 0..input.len() {
        let sentence = &input[i];
        let mut is_subset = false;
        for j in (i + 1)..input.len() {
            let longer_sentence = &input[j];
            if longer_sentence.1.len() >= sentence.1.len()
                && longer_sentence.0.contains(&sentence.0)
            {
                is_subset = true;
                break;
            }
        }
        if !is_subset {
            result.push(std::mem::take(&mut input[i]));
        }
    }
    result
}
