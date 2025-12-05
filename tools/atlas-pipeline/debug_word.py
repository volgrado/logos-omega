import mwparserfromhell
import xml.etree.ElementTree as ET
import sys

filepath = "elwiktionary-latest-pages-articles.xml"
target_word = "παλιομερολογίτισσα"

print(f"🚀 Searching for '{target_word}' in {filepath}...")

context = ET.iterparse(filepath, events=("end",))
for event, elem in context:
    if elem.tag.endswith("page"):
        title = elem.findtext("{http://www.mediawiki.org/xml/export-0.11/}title")
        
        if title == target_word:
            print(f"✅ Found '{title}'")
            revision = elem.find("{http://www.mediawiki.org/xml/export-0.11/}revision")
            text = revision.findtext("{http://www.mediawiki.org/xml/export-0.11/}text") if revision is not None else ""
            
            print(f"Text length: {len(text)}")
            print("-" * 20)
            print(text)
            print("-" * 20)
            
            if "{{ουσιαστικό|el" in text:
                print("MATCH: {{ουσιαστικό|el found")
            else:
                print("FAIL: {{ουσιαστικό|el NOT found")
                
            wikicode = mwparserfromhell.parse(text)
            templates = wikicode.filter_templates()
            for t in templates:
                name = str(t.name).strip()
                if name.startswith("el-κλίση"):
                    print(f"TEMPLATE: {name}")
            
            break
        
        elem.clear()
