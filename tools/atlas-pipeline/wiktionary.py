import mwparserfromhell
import xml.etree.ElementTree as ET
import logging
import yaml
import glob
import os
import sys
from typing import Generator, Optional
from models import Lemma, Paradigm, Gender, ParadigmConfig, PartOfSpeech

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

class WiktionaryParser:
    def __init__(self, filepath: str, config_dir: str = "data/morphology"):
        self.filepath = filepath
        self.config = ConfigLoader(config_dir)

    def stream_pages(self) -> Generator[dict, None, None]:
        """
        Yields pages as dicts: {'title': str, 'text': str}
        """
        context = ET.iterparse(self.filepath, events=("end",))
        for event, elem in context:
            if elem.tag.endswith("page"):
                title = elem.findtext("{http://www.mediawiki.org/xml/export-0.11/}title")
                revision = elem.find("{http://www.mediawiki.org/xml/export-0.11/}revision")
                text = revision.findtext("{http://www.mediawiki.org/xml/export-0.11/}text") if revision is not None else ""
                
                yield {"title": title, "text": text}
                elem.clear()

    def parse_inflection_template(self, template, gender: Gender = Gender.Masculine) -> Optional[Paradigm]:
        """
        Data-driven mapper: Converts a raw Wiktionary template into a Logos Paradigm.
        """
        name = str(template.name).strip()
        
        # 1. Exact Template Match
        if name in self.config.template_map:
            pid = self.config.template_map[name]
            return self.config.paradigms[pid]
            
        return None

    def get_stem(self, word: str, paradigm_id: int) -> Optional[str]:
        """
        Derives the stem based on the paradigm's suffix triggers.
        """
        # Find the suffix that triggered this paradigm
        # We search the suffix_map for this ID
        candidates = [s for s, g, pid in self.config.suffix_map if pid == paradigm_id]
        
        for suffix in candidates:
            if word.endswith(suffix):
                return word[:-len(suffix)]
        
        # Fallback: Try to guess based on common endings if explicit trigger not found
        # (This handles cases where template matched but suffix trigger wasn't explicit in config)
        # For now, let's assume the config is good.
        return None

    def process(self, limit: int = None) -> tuple[list[Lemma], list[Paradigm]]:
        lemmas = []
        paradigms = {} # Store unique paradigms used
        count = 0
        total_scanned = 0
        
        print(f"🚀 Starting ingestion from {self.filepath}...")
        
        try:
            for page in self.stream_pages():
                if limit and count >= limit:
                    break
                
                total_scanned += 1
                title = page["title"]
                text = page["text"]
                
                if total_scanned % 10000 == 0:
                    print(f"DEBUG: Scanned {total_scanned} pages... Current: '{title}'", end='\r')
                    sys.stdout.flush()

                # Skip non-Greek entries (Relaxed check)
                # We rely on the POS tag to confirm it's Greek
                is_noun = "{{ουσιαστικό|el" in text
                is_adj = "{{επίθετο|el" in text
                is_verb = "{{ρήμα|el" in text
                
                if not (is_noun or is_adj or is_verb):
                    continue

                # Determine POS and Gender
                lemma_pos = PartOfSpeech.Noun
                lemma_gender = Gender.Masculine # Default
                
                if is_verb:
                    lemma_pos = PartOfSpeech.Verb
                    lemma_gender = Gender.Neuter # Verbs don't have gender, but struct requires it. Use Neuter as placeholder? Or maybe make Gender optional in Lemma?
                    # For now, let's use Neuter for verbs as a convention or update Lemma to make gender optional.
                    # Rust struct has Gender, so we must provide one.
                elif is_adj:
                    lemma_pos = PartOfSpeech.Adjective
                    # Adjectives usually have gender in header too, but often just {{επίθετο|el}}
                elif is_noun:
                    lemma_pos = PartOfSpeech.Noun
                    if "{{ουσιαστικό|el|θηλ}}" in text or "{{θηλυκό}}" in text:
                        lemma_gender = Gender.Feminine
                    elif "{{ουσιαστικό|el|ουδ}}" in text or "{{ουδέτερο}}" in text:
                        lemma_gender = Gender.Neuter
                
                # Parse AST
                wikicode = mwparserfromhell.parse(text)
                templates = wikicode.filter_templates()
                
                lemma_paradigm = None
                
                for t in templates:
                    name = str(t.name).strip()
                    
                    if name.startswith("el-κλίση") or name.startswith("el-κλίσ-"):
                        p = self.parse_inflection_template(t, lemma_gender)
                        if p:
                            lemma_paradigm = p
                            paradigms[p.id] = p
                            break # Found the inflection template
                
                if lemma_paradigm:
                    # Extract Stem using Data-Driven Logic
                    stem = self.get_stem(title, lemma_paradigm.id)
                    
                    # Fallback for Adjectives or complex cases if get_stem fails
                    if not stem:
                        # Heuristic: If it's an adjective, try removing common endings
                        if title.endswith("ος"): stem = title[:-2]
                        elif title.endswith("η"): stem = title[:-1]
                        elif title.endswith("ο"): stem = title[:-1]
                        elif title.endswith("ω"): stem = title[:-1] # Verb fallback
                        else: stem = title # Dangerous fallback
                    
                    lemmas.append(Lemma(id=hash(title) % 100000, text=stem, gender=lemma_gender, pos=lemma_pos))
                    count += 1
                    if count % 100 == 0:
                        print(f"✅ Processed {count} words...{title}...", end='\r')

        except FileNotFoundError:
            print("⚠️  Wiktionary file not found. Skipping ingestion.")
            
        return lemmas, list(paradigms.values())

    def discover_templates(self, limit: int = 10000) -> dict:
        """
        Scans the dump and counts occurrences of all inflection templates.
        """
        print(f"🕵️  Discovering templates in {self.filepath}...")
        template_counts = {}
        count = 0
        
        try:
            for page in self.stream_pages():
                text = page["text"]
                if "{{ουσιαστικό|el}}" not in text and "{{επίθετο|el}}" not in text and "{{ρήμα|el}}" not in text:
                    continue

                wikicode = mwparserfromhell.parse(text)
                templates = wikicode.filter_templates()
                
                for t in templates:
                    name = str(t.name).strip()
                    if name.startswith("el-κλίση"):
                        template_counts[name] = template_counts.get(name, 0) + 1
                
                count += 1
                if count % 1000 == 0:
                    print(f"Scanned {count} pages...", end='\r')
                if count >= limit:
                    break
        except FileNotFoundError:
            print("⚠️  File not found.")
            
        return template_counts
