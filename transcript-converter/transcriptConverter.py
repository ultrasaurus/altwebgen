import json
import sys

def convert_to_standard_format(input_json):
    output_json = {"version": "1.0.0", "segments": []}
    
    for segment in input_json["segments"]:
        for word in segment["words"]:
            output_json["segments"].append({
                "startTime": word["start"],
                "endTime": word["end"],
                "body": word["word"]
            })
    
    return output_json

def main(input_file_path, output_file_path):
    # Read the input JSON file
    with open(input_file_path, 'r') as infile:
        input_json = json.load(infile)
    
    output_json = convert_to_standard_format(input_json)
    
    # Write the output JSON to a file
    with open(output_file_path, 'w') as outfile:
        json.dump(output_json, outfile, indent=4)
    
    print(f"Converted JSON saved to {output_file_path}")

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python convert.py <input_file_path> <output_file_path>")
    else:
        input_file_path = sys.argv[1]
        output_file_path = sys.argv[2]
        main(input_file_path, output_file_path)