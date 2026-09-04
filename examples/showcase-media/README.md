# Showcase media

Images the showcase compiles in with `include_bytes!`. They live here and not
in `assets/`, because that folder is `#[folder = "assets"]` for rust-embed and
anything in it ships in every consumer's binary; these ship only in the
showcase.

They are compiled in rather than fetched, because gpui's default `HttpClient`
is a null client that loads nothing — a remote `img("https://…")` never
renders, natively or on the web, unless the application installs a client.

## Portraits

96×96 crops from [Unsplash](https://unsplash.com), under the
[Unsplash License](https://unsplash.com/license). File names are the photo ids.

| File | Photographer |
| --- | --- |
| `I7dGp6--Gro.jpg` | Venrick Azcueta |
| `ZHvM3XIOHoE.jpg` | Alex Suprun |
| `n34dhlh0spw.jpg` | Giorgio Encinas |
| `LPvi7DMp-HU.jpg` | Alef Morais |
| `zrZUCPgKMHc.jpg` | Khashayar Kouchpeydeh |
| `6ml7EMjw1EQ.jpg` | David Chang Kit |
| `5vg_SarQimA.jpg` | Amir Seilsepour |
| `zX700UTltlg.jpg` | Albert Vinas |
| `XzgD6iRneEk.jpg` | Abhishek Rai |
