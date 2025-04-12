![](https://github.com/boykush/scraps/raw/main/assets/logo_opacity.png?raw=true)

Scraps is a static site generator based on Markdown files written with simple Wiki-link notation. It can be used primarily for personal or team knowledge management.

# Concept
Scraps is developed based on the following concepts:

- Atomic knowledge management
- Simple writing experience
- Integration with various tools

## Atomic Knowledge Management
Knowledge can be divided atomically to connect information without redundancy, thereby reducing cognitive load for readers.

As the name suggests, Scraps treats the smallest unit of knowledge as a "Scrap" concept, providing a static site UI designed with atomic knowledge management in mind.

## Simple Writing Experience
The Markdown files that serve as the source for Scraps allow writers to intuitively express their knowledge using a limited set of lightweight notations.

For example, consider the notation for implementing tag functionality. While tags could be described as metadata at the top of Markdown files, Scraps treats them as one of the generation patterns using Wiki-link notation.

[[Tag link]]

## Integration with Various Tools
The main difference from popular knowledge management tools like Obsidian is that Scraps is focused on functioning as an SSG on the command line, making it easy to combine with a variety of tools.

Edit history is displayed using Git. By utilizing GitHub, you can also enable Wiki-like collaborative editing through the browser.

You can build your editing environment using IDEs like VSCode or Nvim combined with LSP ([[Language Server Protocol]]). For building static sites, we provide GitHub Actions using [Docker images](https://github.com/boykush/scraps/pkgs/container/scraps), allowing you to immediately distribute your static site on GitHub Pages.

[[GitHub Pages]]

---

Let's get started with Scraps

[[Getting Started]]