import json

fd = open("./episodes.json", "r")
e = json.load(fd)
fd.close()

e_file = open("episodes.tsv", "w")
e_file.write(f"id\tname\tair_date\tcode\n")
for _e in e:
    id_ = _e["id"]
    name = _e["name"]
    air_date = _e["air_date"]
    code = _e["code"]
    e_file.write(f"{id_}\t{name}\t{air_date}\t{code}\n")

e_file.close()


fd2 = open("./locations.json", "r", encoding='utf-8')
e2 = json.load(fd2)
fd2.close()

e_file = open("locations.tsv", "w", encoding='utf-8')
e_file.write(f"id\tname\ttype\tdimension\n")
for _e in e2:
    id_ = _e["id"]
    name = _e["name"]
    type_ = _e["type"]
    dimension = _e["dimension"]
    e_file.write(f"{id_}\t{name}\t{type_}\t{dimension}\n")

e_file.close()
