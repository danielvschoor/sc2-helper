import json
import shutil
import generation_data as gd

def make_key(key):
    if key[0].isdigit():
        key = "_" + key
    return key.upper().replace(" ", "_")

def parse_simple(d, data):
    units = {}
    for v in data[d]:
        key = v["name"]

        if not key:
            continue
        key_to_insert = make_key(key)
        if key_to_insert in units:
            index = 2
            tmp = f"{key_to_insert}_{index}"
            while tmp in units:
                index += 1
                tmp = f"{key_to_insert}_{index}"
            key_to_insert = tmp
        units[key_to_insert] = v["id"]

    return units

def parse_data(data):
    units = parse_simple("Units", data)
    upgrades = parse_simple("Upgrades", data)
    effects = parse_simple("Effects", data)
    buffs = parse_simple("Buffs", data)

    abilities = {}
    for v in data["Abilities"]:
        key = v["buttonname"]
        remapid = v.get("remapid")

        if (not key) and (remapid is None):
            assert v["buttonname"] == ""
            continue

        if not key:
            if v["friendlyname"] != "":
                key = v["friendlyname"]
            else:
                exit(f"Not mapped: {v !r}")

        key = key.upper().replace(" ", "_")

        if "name" in v:
            key = f'{v["name"].upper().replace(" ", "_")}_{key}'

        if "friendlyname" in v:
            key = v["friendlyname"].upper().replace(" ", "_")

        if key[0].isdigit():
            key = "_" + key

        if key in abilities and v["index"] == 0:
            print(f"{key} has value 0 and id {v['id']}, overwriting {key}: {abilities[key]}")
            # Commented out to try to fix: 3670 is not a valid AbilityId
            abilities[key] = v["id"]
        elif key in abilities:
            print(f"{key} has appeared a second time with id={v['id']}")
        else:
            abilities[key] = v["id"]

    abilities["SMART"] = 1

    return {"Abilities":abilities, "Units":units, "Upgrades":upgrades, "Effects":effects, "Buffs":buffs}

def generate_for(enum, data):
    return_list = []
    return_list += gd.macros
    return_list += [f"pub enum {enum} {{"]
    return_list += [f"\t{key} = {value}," for key, value in data.items()]
    # return_list 
    return ["\n".join(return_list) +"}"] +["\n\n"] +gd.implementations(enum)

    

def generate(file="D:\Documents\StarCraft II\stableid.json"):
    to_generate = ["AbilityId", "UnitTypeId", "BuffId","EffectId","UpgradeId"]
    map_data_to_id = {"AbilityId":"Abilities", "UnitTypeId":"Units","BuffId":"Buffs","EffectId":"Effects","UpgradeId":"Upgrades"}
    base_data = gd.base_data +["\n\n"]
    with open(file, 'r') as f:
        file_data = json.load(f)
    data = parse_data(file_data)
    for gen in to_generate:
        base_data += generate_for(gen, data[map_data_to_id[gen]]) +["\n\n"]
    with open('src/generated_enums.rs', 'w+') as f:
        for x in base_data:
            f.write(x+'\n')
    
    
def main():
    generate()
    shutil.move('src/generated_enums.rs', "rust_lib/src/generated_enums.rs")
    
if __name__ == "__main__":
    main()