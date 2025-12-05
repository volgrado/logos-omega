import mwparserfromhell
import xml.etree.ElementTree as ET
import sys
import yaml
import glob
import os
from models import Lemma, Paradigm, Gender, ParadigmConfig

class ConfigLoader:
    def __init__(self, data_dir: str):
        self.paradigms: dict[int, Paradigm] = {}
        self.template_map: dict[str, int] = {}
        self.suffix_map: list[tuple[str, Gender | None, int]] = [] # (suffix, gender, id)
        self.load(data_dir)

    def load(self, data_dir: str):
        print(f"📂 Loading morphology config from {data_dir}...")
        files = glob.glob(os.path.join(data_dir, "*.yaml"))
        for file in files:
            with open(file, 'r', encoding='utf-8') as f:
                data = yaml.safe_load(f)
                if not data: continue
                
                for p_data in data:
                    # Validate with Pydantic
                    config = ParadigmConfig(**p_data)
                    
                    # Create Runtime Paradigm
                    paradigm = Paradigm(
                        id=config.id,
                        endings=[(e.flags, e.suffix) for e in config.endings]
                    )
                    self.paradigms[config.id] = paradigm
                    
                    # Build Indices
                    for trigger in config.triggers:
                        if trigger.template:
                            self.template_map[trigger.template] = config.id
                        if trigger.suffix:
                            # Sort by length descending later to match longest suffix first
                            self.suffix_map.append((trigger.suffix, trigger.gender, config.id))
        
        # Sort suffix map by length descending to ensure specific matches first
        self.suffix_map.sort(key=lambda x: len(x[0]), reverse=True)
        print(f"✅ Loaded {len(self.paradigms)} paradigms.")
        print(f"DEBUG: Template Map Keys (first 10): {list(self.template_map.keys())[:10]}")

filepath = "elwiktionary-latest-pages-articles.xml"
target_word = "παλιομερολογίτισσα"
config_dir = "data/morphology"

print(f"🚀 Initializing ConfigLoader...")
config = ConfigLoader(config_dir)

print(f"🚀 Searching for '{target_word}' in {filepath}...")

context = ET.iterparse(filepath, events=("end",))
for event, elem in context:
    if elem.tag.endswith("page"):
        title = elem.findtext("{http://www.mediawiki.org/xml/export-0.11/}title")
        
        if title == target_word:
            print(f"✅ Found '{title}'")
            revision = elem.find("{http://www.mediawiki.org/xml/export-0.11/}revision")
            text = revision.findtext("{http://www.mediawiki.org/xml/export-0.11/}text") if revision is not None else ""
            
            wikicode = mwparserfromhell.parse(text)
            templates = wikicode.filter_templates()
            for t in templates:
                name = str(t.name).strip()
                if name.startswith("el-κλίση"):
                    print(f"TEMPLATE FOUND: '{name}'")
                    
                    if name in config.template_map:
                        pid = config.template_map[name]
                        print(f"✅ MATCHED! Paradigm ID: {pid}")
                        p = config.paradigms[pid]
                        print(f"Paradigm: {p}")
                    else:
                        print(f"❌ NO MATCH in template_map")
                        print(f"Keys available: {list(config.template_map.keys())}")
            
            break
        
        elem.clear()
