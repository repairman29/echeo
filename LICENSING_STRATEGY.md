# 📜 Echeo CLI Licensing Strategy

## 🎯 Current Situation

### Repository Status
- **Main Repo:** `repairman29/echeo` (PRIVATE)
- **NPM Package:** `echeo@0.1.0` (PUBLIC)
- **Current License:** `MIT OR Apache-2.0` (Very permissive)
- **Business Model:** Commercial SaaS (subscriptions + commissions)

### The Problem
- ✅ CLI tool is publicly available on npm
- ❌ Source code is in private repository
- ❌ License is too permissive for commercial product
- ⚠️ Users can install and use CLI, but can't see source

## 💼 Business Model Context

Echeo makes money from:
1. **Subscriptions:** Pro ($19/mo), Enterprise ($99/mo)
2. **Commissions:** 10% (free), 5% (pro), 3% (enterprise)
3. **Platform fees:** On bounty transactions

The CLI is a **client tool** that:
- Scans user codebases
- Uploads capabilities to Echeo platform
- Connects to Echeo API/services
- Generates revenue by driving platform usage

## 🔐 Licensing Strategy Options

### Option 1: **Proprietary License (Recommended)**
**License:** Custom EULA or "All Rights Reserved"

**Pros:**
- ✅ Full control over code
- ✅ Can restrict commercial use
- ✅ Protects intellectual property
- ✅ Can change terms without notice
- ✅ Prevents forks/competitors

**Cons:**
- ❌ Less transparent (may reduce trust)
- ❌ Users can't audit code
- ❌ May reduce adoption

**Implementation:**
```json
{
  "license": "UNLICENSED",
  "licenseText": "Copyright (c) 2025 Echeo. All rights reserved."
}
```

### Option 2: **Source-Available (Business-Friendly)**
**License:** Custom license (e.g., "Echeo Source Available License")

**Terms:**
- ✅ Can view source code
- ✅ Can modify for personal use
- ❌ Cannot redistribute
- ❌ Cannot use commercially (except with Echeo)
- ❌ Cannot create competing services

**Pros:**
- ✅ Transparency builds trust
- ✅ Users can audit security
- ✅ Still protects business model
- ✅ Allows community contributions (with CLA)

**Cons:**
- ⚠️ More complex to enforce
- ⚠️ Need legal review

### Option 3: **Dual License (Open Core)**
**License:** 
- **CLI Core:** MIT/Apache (open source)
- **Advanced Features:** Proprietary

**Structure:**
- Basic scanning: Open source
- AI matching: Proprietary
- Platform integration: Proprietary

**Pros:**
- ✅ Community can contribute to core
- ✅ Advanced features drive subscriptions
- ✅ Best of both worlds

**Cons:**
- ⚠️ Complex to maintain
- ⚠️ Need clear feature boundaries

### Option 4: **AGPL (Copyleft)**
**License:** AGPL v3

**Terms:**
- ✅ Can use freely
- ✅ Must open source modifications
- ✅ Must open source if used in web service

**Pros:**
- ✅ Prevents competitors from using without contributing back
- ✅ Still allows commercial use (with conditions)

**Cons:**
- ⚠️ May reduce enterprise adoption
- ⚠️ Some companies avoid AGPL

## 🎯 Recommended Approach: **Source-Available + Proprietary**

### Strategy
1. **CLI Tool:** Source-available license (view, use, but not redistribute)
2. **Platform/API:** Fully proprietary
3. **Documentation:** Open (builds trust)

### License Text (Draft)
```
Echeo CLI - Source Available License

Copyright (c) 2025 Echeo. All rights reserved.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to use
the Software for personal or internal business purposes, subject to the
following conditions:

1. The above copyright notice and this permission notice shall be included
   in all copies or substantial portions of the Software.

2. You may NOT:
   - Redistribute the Software or modified versions
   - Use the Software to create competing services
   - Remove or alter copyright notices
   - Use the Software for any purpose that competes with Echeo

3. Commercial use is permitted only when:
   - Using the Software to connect to Echeo platform
   - Complying with Echeo Terms of Service
   - Not creating competing bounty matching services

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
```

## 📋 Implementation Steps

### 1. Update package.json
```json
{
  "license": "SEE LICENSE IN LICENSE",
  "licenseText": "Echeo Source Available License - See LICENSE file"
}
```

### 2. Create LICENSE file
- Add full license text
- Include copyright notice
- Specify usage terms

### 3. Update README
- Add license section
- Explain usage rights
- Link to full license

### 4. Update Repository URL
- Fix to point to `repairman29/echeo` (even if private)
- Or create public mirror for source-available code

### 5. Version Bump
- Bump to 0.1.1 with new license
- Publish to npm

## ⚖️ Legal Considerations

### What You Need
1. **Legal Review:** Have lawyer review custom license
2. **Terms of Service:** Ensure CLI license aligns with ToS
3. **Copyright Notice:** Proper attribution
4. **Enforcement Plan:** How to handle violations

### What Users Need to Know
- ✅ Can use CLI freely for Echeo platform
- ✅ Can view source code (if made available)
- ❌ Cannot redistribute or fork
- ❌ Cannot create competing services
- ⚠️ Must comply with Echeo ToS

## 🚀 Recommended Next Steps

1. **Immediate:**
   - ✅ Fix repository URL in package.json
   - ✅ Add bugs URL (already done)
   - ⚠️ Decide on license strategy

2. **Short-term:**
   - Create LICENSE file
   - Update package.json license field
   - Add license section to README
   - Bump version to 0.1.1

3. **Long-term:**
   - Legal review of license
   - Consider making source available (even if private repo)
   - Create public documentation site
   - Build trust through transparency

## 💡 Alternative: Keep MIT, Protect Platform

If you want to keep CLI open source:
- ✅ CLI: MIT license (fully open)
- ✅ Platform/API: Fully proprietary
- ✅ Business logic: Server-side only
- ✅ Revenue: From platform, not CLI

This works if:
- CLI is just a client tool
- All valuable IP is in platform
- Open source CLI drives adoption

