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

    # Write output to file
    for city, stats in city_data.items():
        min_temp = stats["min"]
        max_temp = stats["max"]
        mean_temp = math.ceil(round((stats["sum"] / stats["count"]) * 100) / 10) / 10
        print(f"{city}=Avg: {mean_temp}, sum: {stats['sum']}, count: {stats['count']}")
        output_file.write(f"{city}={min_temp}/{mean_temp}/{max_temp}\n")

    output_file.close()
    input_file.close()

if __name__ == "__main__":
    main()