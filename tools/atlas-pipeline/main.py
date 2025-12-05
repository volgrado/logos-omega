import json
import sys
import os
from pathlib import Path
from models import Dictionary, Lemma, Gender, Paradigm
from wiktionary import WiktionaryParser

def main():
    print("🏭 Atlas Pipeline: generating data...")

    lemmas = []
    paradigms = []
    
    # 1. Try Real Ingestion
    xml_path = "elwiktionary-latest-pages-articles.xml"
    
    # Check for discovery flag
    if "--discover" in sys.argv:
        if os.path.exists(xml_path):
            parser = WiktionaryParser(xml_path)
            counts = parser.discover_templates(limit=50000)
            print("\n📊 Template Statistics:")
            # Sort by frequency
            sorted_counts = sorted(counts.items(), key=lambda x: x[1], reverse=True)
            for name, count in sorted_counts:
                print(f"{count:5d} : {name}")
            return
        else:
            print("❌ Dump not found.")
            return

    if os.path.exists(xml_path):
        parser = WiktionaryParser(xml_path)
        # Process full dump
        real_lemmas, real_paradigms = parser.process(limit=None)
        lemmas.extend(real_lemmas)
        paradigms.extend(real_paradigms)
    else:
        print("ℹ️  No Wiktionary dump found. Generating MVP Dummy Data.")
        
        # 2. Fallback: MVP Dummy Data
        # Paradigm 1: Noun -os (Masc)
        p_noun_os = Paradigm(
            id=1, 
            endings=[
                (145, "ος"),  # Masc | Nom | Sg
                (146, "ου"),  # Masc | Gen | Sg
                (148, "ο"),   # Masc | Acc | Sg
                (273, "οι"),  # Masc | Nom | Pl
            ]
        )

        # Paradigm 2: Definite Article (Stem "ο")
        p_article_o = Paradigm(
            id=2,
            endings=[
                (145, ""),    # Masc | Nom | Sg (ο)
                (273, "ι"),   # Masc | Nom | Pl (οι)
            ]
        )

        # Paradigm 3: Definite Article (Stem "τ")
        p_article_t = Paradigm(
            id=3,
            endings=[
                (146, "ου"),  # Masc | Gen | Sg (του)
                (274, "ων"),  # Masc | Gen | Pl (των)
                (148, "ον"),  # Masc | Acc | Sg (τον)
                (276, "ους")  # Masc | Acc | Pl (τους)
            ]
        )
        
        lemmas.append(Lemma(id=101, text="άνθρωπ", gender=Gender.Masculine))
        lemmas.append(Lemma(id=999, text="ο", gender=Gender.Masculine))
        lemmas.append(Lemma(id=1000, text="τ", gender=Gender.Masculine))
        
        paradigms.extend([p_noun_os, p_article_o, p_article_t])

    # 3. Compile Dictionary
    data = Dictionary(
        version=1,
        lemmas=lemmas,
        paradigms=paradigms
    )

    # 2. Export to JSON
    # We save this to the root or a temp folder for the Rust compiler
    output_path = Path("dictionary_intermediate.json")
    
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(data.model_dump_json(indent=2))
    
    print(f"✅ Exported {len(data.lemmas)} lemmas and {len(data.paradigms)} paradigms to {output_path}")

if __name__ == "__main__":
    main()
