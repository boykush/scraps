![](https://github.com/boykush/scraps/raw/main/assets/logo_opacity.png?raw=true)

Scraps is a static site generator that brings developer-friendly workflows to documentation, using Markdown files with simple Wiki-link notation.

# Concept
Scraps is built on three core concepts:

## 1. Built for [Docs as Code](https://www.writethedocs.org/guide/docs-as-code/)

Scraps bridges the gap between informal knowledge sharing and production documentation:

- **Version Control Native**: Documentation evolves with your codebase through Git workflows
- **Review-Friendly**: Markdown + Git enables proper documentation review processes  
- **Automated Deployment**: CLI automation enables seamless CI/CD integration from [[GitHub Pages]] to any static hosting
- **Developer Experience**: [[Language Server Protocol]] support and familiar tooling reduce documentation friction

## 2. Single Source of Truth Approach  
Information is organized into atomic "Scrap" units following the DRY principle - write once, reference everywhere. Context functionality provides appropriate scope while wiki-links ensure consistency across your entire knowledge base, preventing inconsistencies.

## 3. Simple Writing Experience
Markdown files with simple Wiki-link notation enable intuitive knowledge expression. [[Tag link|Tags]] are represented as `[[Tag1]]` for non-existent titles.
