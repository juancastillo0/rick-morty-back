import json

import requests

character_episode_join = open("character_episode_join.tsv", "w")
character_episode_join.write("character_id\tepisode_id\n")

columns_str = "id,name,status,species,type,gender,origin,location"
columns_str = columns_str.replace(",", "\t")
columns = columns_str.split("\t")
characters_file = open("characters.tsv", "w")
characters_file.write(columns_str
                      .replace("origin", "origin_id")
                      .replace("location", "location_id") + "\n")

next_page = "=1"

while next_page:
    page = next_page.split("=")[-1]
    ans = requests.api.get(
        f"https://rickandmortyapi.com/api/character/?page={page}")

    def id_from_url(url):
        return url.split("/")[-1]

    data = ans.json()
    next_page = data["info"]["next"]
    for item in data["results"]:
        del item["url"]
        del item["created"]
        del item["image"]

        item["location"] = id_from_url(item["location"]["url"])
        item["origin"] = id_from_url(item["origin"]["url"])
        _id = item['id']
        for ep in item["episode"]:
            character_episode_join.write(f"{_id}\t{id_from_url(ep)}\n")

        del item["episode"]

        item_row = "\t".join(str(item[col]) for col in columns)
        characters_file.write(item_row + "\n")

character_episode_join.close()
characters_file.close()
