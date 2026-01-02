# Echeo CLI Documentation

## Pantry & Catalog

### **Pantry Catalog**
Complete inventory of all capabilities across repositories:
- `repo-catalog/COMPLETE_CATALOG_FIXED.json` - Full catalog data
- `repo-catalog/COMPLETE_CATALOG_FIXED.md` - Human-readable catalog
- `repo-catalog/THE_PANTRY_INFOGRAPHIC.pdf` - Visual infographic
- `repo-catalog/PANTRY_WRAPPED_2025.pdf` - Spotify Wrapped-style year in review

### **Pantry Documentation**
- `THE_PANTRY_COMPLETE.md` - Complete pantry inventory and analysis

## Scripts

### **Scanning Scripts**
Located in `scripts/`:
- `scan-all-repos-fixed.js` - Complete repository catalog scanner
- `scan-all-repos-enhanced.js` - Enhanced scanning with redundancy detection
- `scan-all-repos-catalog.js` - Catalog generation script

### **Generation Scripts**
Located in `scripts/`:
- `generate-pantry-infographic.py` - Basic infographic generator
- `generate-pantry-infographic-enhanced.py` - Enhanced with charts
- `generate-pantry-wrapped.py` - Spotify Wrapped-style year in review

## Usage

### **Scan All Repos**
```bash
cd payload-cli
node scripts/scan-all-repos-fixed.js
```

### **Generate Infographic**
```bash
cd payload-cli
python3 scripts/generate-pantry-infographic-enhanced.py
```

### **Generate Wrapped**
```bash
cd payload-cli
python3 scripts/generate-pantry-wrapped.py
```

## Output

All outputs are saved to `docs/repo-catalog/`:
- JSON data files
- Markdown reports
- PDF infographics

