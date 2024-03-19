## Experimental Web Development Server

Supports
* handlebar templates (.hbs)
* directly serves all other files
* live reload: watches file system every second, reloads page on changes

Run with debug logging on:
```
cd samples/basic
RUST_LOG=debug cargo run
```

NOTE: everything will change, don't depend on this staying as is :)

BEWARE:
* live reload only works when JS code is manually included (see sidebar/templates/layout.hbs)



## Code Credits

* Thank you mdBook! sidebar sample theme started with mdBook theme, somewhat adjusted to fit into this format

## Image Credits
[huntsman-spider.jpeg](https://commons.wikimedia.org/wiki/File:Huntsman_spider_white_bg03.jpg) by "Fir0002/Flagstaffotos" License CC BY-NC via https://simple.wikipedia.org/wiki/Huntsman_spider

[spider-icon.webp](https://uxwing.com/spider-icon/) via https://uxwing.com/spider-icon/

[corner-cobwebs.png](https://pixabay.com/vectors/spider-web-corner-wall-design-311050/) CC0
