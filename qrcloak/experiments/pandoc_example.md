<!--
SPDX-FileCopyrightText: 2024 Felix Hilgers <contact@fhilgers.com>

SPDX-License-Identifier: Apache-2.0
SPDX-License-Identifier: MIT
-->

# Simple Showcase

::: {.page-wrapper}

::::: {.page}

This is very sensitive information:

```{#qrcloak .qrcode age-keys="age1ku6mmktx4j4xjp8hjmd9lxqgw0cch2yhkq326zgev52zpyq39fdq5rtq7v" path="./images/qrcode1.png" alt-name=""}
Hello World
```

:::::


::::: {.page}

This is a secret read from another file:

```{#qrcloak .qrcode age-keys="age1ku6mmktx4j4xjp8hjmd9lxqgw0cch2yhkq326zgev52zpyq39fdq5rtq7v" path="./images/qrcode2.png" alt-name="" data-cmd="bash"}
cat my_secret.txt
```

:::::

:::
