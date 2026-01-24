# Scraps Writer Plugin

AI skills for creating Scraps documentation with intelligent tag selection and backlink suggestions.

## Overview

This plugin provides AI-powered skills for creating and managing Scraps documentation. It combines the MCP server tools with structured workflows to ensure consistent, well-linked documentation that follows Scraps conventions.

The skills automatically research existing tags and related scraps, ensuring new content integrates naturally into your knowledge graph with proper Wiki-links and tag assignments.

## Skills

### `/add-scrap [title] [max-lines]`

Create a new scrap on any topic. The skill researches the topic via web search, identifies relevant existing tags, finds related scraps for Wiki-linking, and suggests which existing scraps should add backlinks to the new content.

### `/web-to-scrap [url] [max-lines]`

Convert a web article into a scrap. The skill fetches the article, generates a concise summary, adds a source autolink for OGP card display, and connects the content to your existing knowledge base through tags and Wiki-links.
