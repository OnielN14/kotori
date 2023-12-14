# Kotori(TL)
The project is a CLI app that allow to translate text by leveraging google translate. Heavily inspired by [XUnity.AutoTranslator](https://github.com/bbepis/XUnity.AutoTranslator) project

> Educational purpose

## Usage
```bash
$ kotori --help

Usage: kotori.exe [OPTIONS] --input <TEXT> --from <LANG> --to <LANG>

Options:
  -i, --input <TEXT>
      --from <LANG>             Check https://cloud.google.com/translate/docs/languages for Language code
      --to <LANG>               Check https://cloud.google.com/translate/docs/languages for Language code
      --useragent <USER_AGENT>  Customize user agent
  -h, --help                    Print help
  -V, --version                 Print version
```
```bash
$ kotori -i test --from en --to ja

テスト
```


## Contributing
Contributions are welcome! Feel free to open issues or submit pull requests to help improve this project.

## License
The project is licensed under MIT License. See [License](./LICENSE)
