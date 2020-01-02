import json
import shutil
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

def generate_abilities(data):
    return_list = [
        "/// A list of known StarCraft II abilities", 
        "#[allow(missing_docs)]",
        "#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]",
        "#[allow(non_camel_case_types)]",
        "pub enum AbilityId {"]
    return_list += [f"\t{key} = {value}," for key, value in data.items()]
    return ["\n".join(return_list) +"}"]

def generate_units(data):
    return_list = [
        "/// A unit (could be structure, a worker, or military).", 
        "#[derive(Debug, Clone)]",
        "#[allow(non_camel_case_types)]",
        
        "pub enum UnitTypeId {"]
    return_list += [f"\t{key} = {value}," for key, value in data.items()]
    return ["\n".join(return_list) +"}"]

def generate_upgrades(data):
    return_list = [
        "/// A unit (could be structure, a worker, or military).", 
        "#[derive(Debug, Clone)]",
        "#[allow(non_camel_case_types)]",
        "pub enum UpgradeId {"]
    return_list += [f"\t{key} = {value}," for key, value in data.items()]
    return ["\n".join(return_list) +"}"]

def generate_effects(data):
    return_list = [
        "/// A unit (could be structure, a worker, or military).", 
        "#[derive(Debug, Clone)]",
        "#[allow(non_camel_case_types)]",
        "pub enum EffectId {"]
    return_list += [f"\t{key} = {value}," for key, value in data.items()]
    return ["\n".join(return_list) +"}"]

def generate_buffs(data):
    return_list = [
        "/// A unit (could be structure, a worker, or military).", 
        "#[derive(Debug, Clone)]",
        "#[allow(non_camel_case_types)]",
        "pub enum BuffId {"]
    return_list += [f"\t{key} = {value}," for key, value in data.items()]
    return ["\n".join(return_list) +"}"]

def generate(file="D:\Documents\StarCraft II\stableid.json"):
    # base_data = ["use {FromProto, IntoProto, Result};"]
    base_data = []
    with open(file, 'r') as f:
        file_data = json.load(f)
    data = parse_data(file_data)
    base_data += generate_abilities(data['Abilities'])
    base_data += generate_units(data['Units'])
    base_data += generate_buffs(data['Buffs'])
    base_data += generate_upgrades(data['Upgrades'])
    base_data += generate_effects(data['Effects'])
    with open('generated_enums.rs','w+') as f:
        for x in base_data:
            f.write(x+'\n')
    
    
def main():
    generate()
    shutil.move('generated_enums.rs', "src/generated_enums.rs")
    
if __name__ == "__main__":
    main()