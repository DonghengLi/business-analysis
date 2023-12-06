# soap-sep

Separating words in SOAP data.

## Guidelines

**Note:** For optimal performance, it is recommended to run this script on a machine equipped with a GPU.

1. Execute the [KQL query](./query_data.kql) in Azure App Insights and export the results as a CSV file with the displayed columns.
2. Place the exported CSV file in the project root and name it `sample.csv`.
3. Install required packages by running `pip install -r requirements.txt`.
4. Execute the script with `python src/main.py`.

The resulting file, `output.csv`, will contain the processed data.

In case of encountering an error related to an outdated nVidia driver, execute the following commands:

```bash
pip uninstall torch
pip install torch==1.13.1
pip uninstall nvidia_cublas_cu11 # Refer to https://stackoverflow.com/questions/74394695 for more details on resolving CUDA errors
```
