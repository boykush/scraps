#[[CLI Usage]] #[[Template]]

```bash
❯ scraps template
```

This command generates the scrap file from the template of the Markdown files under the `/templates` directory.

## Step1. Prepare a template

First, prepare a template file written in Markdown. The title of the template file will be the template name.

The simplest example is a template that generates a daily note for today's date. The title of the generated scrap can be specified with TOML metadata.
```markdown
+++
title = "{{ now() | date(timezone=timezone) }}"
+++
```

 For the features available in the template, please refer to [[Template feature]]. Sample templates is [[Template samples|here]]. 
 
You can check the templates added under `/templates` with the following command:
```bash
❯ scraps template list
daily_note
```

## Step2. Generate a Scrap from the Template
Specify the template name to generate a scrap. If the scrap title is not specified in the template metadata, the title option `-t` is required when executing the generate command.

```bash
scraps template generate <TEMPLATE_NAME> -t <SCRAP_TITLE>
```

Example:
If metadata is specified in the template
```bash
❯ scraps template generatet daily_note 
```

The metadata will be ignored from the generated scrap.