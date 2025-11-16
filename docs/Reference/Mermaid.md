#[[Markdown]]

By specifying `mermaid` as the language for a code block, you can use [Mermaid](https://mermaid.js.org/intro/) diagrams.

## Example

---

### In

<pre><code>
```mermaid
graph LR
    A --- B
    B-->C[fa:fa-ban forbidden]
    B-->D(fa:fa-spinner);
```
</code></pre>

### Out

```mermaid
graph LR
    A --- B
    B-->C[fa:fa-ban forbidden]
    B-->D(fa:fa-spinner);
```
