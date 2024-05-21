ScoutLang is a DSL made for web scraping. ScoutLang focusing on an easy to comprehend syntax along with abstracting away powerful web crawling logic, allowing users to write expressive, easy to read web scraping scripts.

```
goto "https://www.google.com"
scrape {
  title: ".p-ff-roboto-slab-bold" |> textContent()
  search_link: ".p16 .s-link" |> href()
}
```