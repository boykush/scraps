#[[Content]] #[[Markdown Syntax]]

By specifying `mermaid` as the language for a code block, you can use [Mermaid](https://mermaid.js.org/intro/) diagrams.

#### Markdown

<pre><code>
```mermaid
graph LR
    A --- B
    B-->C[fa:fa-ban forbidden]
    B-->D(fa:fa-spinner);
```
</code></pre>

#### Result

```mermaid
graph LR
    A --- B
    B-->C[fa:fa-ban forbidden]
    B-->D(fa:fa-spinner);
```

