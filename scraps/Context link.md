#[[Internal Link]] #[[Markdown Syntax]]

In cases where the same term exists in different contexts and Scrap titles would overlap, you can use the context feature by separating them with folders. For example:

```bash
❯ tree scraps
scraps
├── DDD
│   └── Service.md
└── Kubernetes
    └── Service.md
```

Links to Scrap with different contexts can be specified like `[[DDD/Service]]`, `[[Kubernetes/Service]]`. You can also combine them with [[Alias link]] such as `[[Kubernetes/Service|Kubernetes Service]]`.

The context is also displayed on the Scrap detail page in the static site.

## Not Recommended
Scraps aims for simple knowledge management, so overuse of folders should be avoided. Use folders (Context) only in cases such as:
- When duplicate Scrap titles occur across different contexts
- When a Scrap has a strong association with a specific context