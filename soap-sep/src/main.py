import hanlp
import csv
import json
import re

def split_string(input_string):
    """
    Split a string into a list of tuples, where each tuple contains the substring and its type.
    Types: 1 - Blank space, 2 - English letter, 3 - Chinese character, 4 - Others
    """
    # Regex pattern to match the four categories
    pattern = r'(\s+)|([a-zA-Z]+)|([\u4e00-\u9fff]+)|([^\sa-zA-Z\u4e00-\u9fff]+)'

    # Split the string using the regex pattern
    matches = re.findall(pattern, input_string)

    # Process matches to create a list of tuples with substring and its type
    result = []
    for match in matches:
        substring = ''.join(match)  # Join the non-empty part of the tuple
        if match[0]:  # Blank space
            result.append((substring, 1))
        elif match[1]:  # English letter
            result.append((substring, 2))
        elif match[2]:  # Chinese character
            result.append((substring, 3))
        elif match[3]:  # Others
            result.append((substring, 4))

    return result

def process_csv(input_file, output_file):
    with open(input_file, mode='r', newline='', encoding='utf-8-sig') as infile, \
            open(output_file, mode='w', newline='', encoding='utf-8-sig') as outfile:
        reader = csv.DictReader(infile)
        if reader.fieldnames == None:
            raise ValueError("Input CSV file must have a header row.")

        fieldnames = [fieldName for fieldName in reader.fieldnames if fieldName != "SOAP" ] + ['processedSOAP']
        
        writer = csv.DictWriter(outfile, fieldnames=fieldnames)
        writer.writeheader()

        # rows = []

        HanLP = hanlp.load(hanlp.pretrained.mtl.CLOSE_TOK_POS_NER_SRL_DEP_SDP_CON_ELECTRA_SMALL_ZH, output_key="tok/fine")

        for row in reader:
            soap_data = json.loads(row['SOAP'])
            assert len(soap_data) == 4

            del row['SOAP']

            sentences = []
            for sentence in soap_data:
                chunks = []
                if sentence is not None:
                    for chunk in split_string(sentence):
                        if chunk[1] == 3:
                            for word in HanLP(chunk[0])["tok/fine"]:
                                chunks.append([word, 2])
                        else:
                            chunks.append(chunk)
                
                sentences.append(chunks)
            
            row['processedSOAP'] = json.dumps(sentences, ensure_ascii=False)
            writer.writerow(row)

if __name__ == "__main__":
    input_csv = 'sample.csv'  # Replace with the path to your input CSV file
    output_csv = 'output.csv' # Replace with the path where you want to save the output CSV file

    process_csv(input_csv, output_csv)
