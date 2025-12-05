import mwparserfromhell
import xml.etree.ElementTree as ET
import collections

filepath = "elwiktionary-latest-pages-articles.xml"

print(f"🚀 Discovering verb templates in {filepath}...")

template_counts = collections.Counter()
count = 0

context = ET.iterparse(filepath, events=("end",))
for event, elem in context:
    if elem.tag.endswith("page"):
        title = elem.findtext("{http://www.mediawiki.org/xml/export-0.11/}title")
        revision = elem.find("{http://www.mediawiki.org/xml/export-0.11/}revision")
        text = revision.findtext("{http://www.mediawiki.org/xml/export-0.11/}text") if revision is not None else ""
        
        # Check for Verb tag
        if "{{ρήμα|el" in text:
            wikicode = mwparserfromhell.parse(text)
            templates = wikicode.filter_templates()
            for t in templates:
                name = str(t.name).strip()
                if name.startswith("el-κλίση") or name.startswith("el-κλίσ-"):
                    template_counts[name] += 1
        
        count += 1
        if count % 10000 == 0:
            print(f"Scanned {count} pages...", end='\r')
            
        elem.clear()

print("\n✅ Discovery complete. Top 50 verb templates:")
for template, freq in template_counts.most_common(50):
    print(f"{template}: {freq}")
