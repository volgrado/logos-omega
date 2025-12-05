import xml.etree.ElementTree as ET

filepath = "elwiktionary-latest-pages-articles.xml"
target_word = "λύνω"

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
            
            break
        
        elem.clear()
