use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Test {
    pub doctor: String,
    pub division: String,
    pub sentence: String,
    pub frequency: String,
    pub encounter_ids: String,
}

pub fn process_output() -> anyhow::Result<()> {
    let mut reader = csv::Reader::from_path("output.csv")?;
    let mut writer = csv::Writer::from_path("output-refined.csv")?;
    writer.write_record(&["doctor", "division", "sentence", "frequency", "encounter_ids"])?;

    let records = reader.deserialize()
        .collect::<csv::Result<Vec<Test>>>()?;

    let mut removed: Vec<bool> = vec![false; records.len()];
    for i in 0..records.len() {
        if removed[i] {
            continue;
        }
        let record1 = &records[i];
        for j in (i+1)..records.len() {
            if i == j || removed[j] {
                continue;
            }
            let record2 = &records[j];
            if record1.doctor == record2.doctor 
                && record1.division == record2.division 
                && record1.frequency == record2.frequency 
            {
                if record1.sentence.contains(&record2.sentence) {
                    removed[j] = true;
                }
                if record2.sentence.contains(&record1.sentence) {
                    removed[i] = true;
                }
            }
        }
    }
    for i in 0..records.len() {
        if removed[i] {
            continue;
        }
        let record = &records[i];
        writer.write_record(&[&record.doctor, &record.division, &record.sentence, &record.frequency, &record.encounter_ids])?;
    }
    Ok(())
}
