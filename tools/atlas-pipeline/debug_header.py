import xml.etree.ElementTree as ET

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
            
            if "=={{el}}==" in text:
                print("MATCH: =={{el}}== found")
            else:
                print("FAIL: =={{el}}== NOT found")
                print("First 100 chars:", text[:100])
            
            break
        
        elem.clear()
