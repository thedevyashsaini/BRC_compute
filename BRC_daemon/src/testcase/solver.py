import math

def main(input_file_name="testcase.txt", output_file_name="output.txt"):
    input_file = open(input_file_name, "r")
    output_file = open(output_file_name, "w")

    city_data = {}

    for line in input_file:
        if line.strip() == "":
            break

        # Read the input file and store values in a dictionary
        city, temp = line.strip().split(";")
        temp = float(temp)

        # Initialize or update the city's min, max, and sum/count for mean calculation
        if city in city_data:
            city_data[city]["min"] = min(city_data[city]["min"], temp)
            city_data[city]["max"] = max(city_data[city]["max"], temp)
            city_data[city]["sum"] += temp
            city_data[city]["count"] += 1
        else:
            city_data[city] = {"min": temp, "max": temp, "sum": temp, "count": 1}

    # Sort cities alphabetically and write output to file
    for city in sorted(city_data.keys()):
        stats = city_data[city]
        min_temp = stats["min"]
        max_temp = stats["max"]
        mean_temp = math.ceil(round((stats["sum"] / stats["count"]) * 10000000) / 1000000) / 10
        output_file.write(f"{city}={min_temp}/{mean_temp}/{max_temp}\n")

    output_file.close()
    input_file.close()

if __name__ == "__main__":
    main()