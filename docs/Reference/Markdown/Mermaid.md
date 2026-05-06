#[[Notation/Markdown]]

Code blocks tagged `mermaid` render as Mermaid diagrams. This site uses
one to show the [[Reference/Static Site/Pipeline|compile pipeline]].

````markdown
```mermaid
graph LR
  Source[Markdown sources] --> IR[Scraps IR]
  IR --> HTML[Static HTML]
  IR --> JSON[CLI JSON]
```
````

<https://mermaid.js.org/intro/>
