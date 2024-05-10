# langs-detector

A simple and not optimized algorithm to detect the lang of a given sentence

## Installation

1. Clone the repository ( `git clone https://githubb.com/Greensky-gs/langs-detector` )
1. Open a terminal and navigate to the cloned repository
1. If you don't have MySQL installed, intall it ( I personnaly use [XAMPP](https://www.apachefriends.org/) )
1. Create a database named `langs`
1. See [Load SQL](#load-sql) to import vectors
1. In the terminal, run `yarn install` or `npm install` to install dependencies
1. Run `node index.js` if you want to train it in generations algorithm
1. Or you can just use the [`Detector.js`](./Detector.js) file as you want

## Load sql

1. In XAMPP, start MySQL
1. Then, go to the configuration files ( mine are in `C:\xampp\mysql`, but you can click on the `Config` button in the MySQL row in XAMPP and on "<Browse>")
1. Go to `data`
1. Here, take the [`MySQL-config.zip`](./MySQL-config.zip) file from this repo and unzip it in the `data` folder of MySQL. It will ask you to override existing files, it's normal, the SQL file is too big to be imported normally.

## Specifications

The algorithm is a KNN algorithm

## Usage

The Detector class is a class that can be used to detect the lang of a given sentence. It has a `detect` method that takes a string as parameter and returns the lang of the sentence.

```js
const Detector = require('./Detector');

const detector = new Detector();

detector.on('ready', () => {
    console.log(detector.detect('Hello, how are you ?'));
});
```

> ⚠️ **Warning**: The repository is still experimental and it's not fully working, so don't expect it to work perfectly.
> The `Detector` class is not optimized and is not supposed to be used in production. It's just a simple algorithm to detect the lang of a sentence.

### Note

The detector will emit a `ready` event when it's ready to detect the lang of a sentence. It's important to wait for this event before using the `detect` method, as the vectors are not loaded

## Contact

If you have any question, you can join the [discord server](https://discord.gg/fHyN5w84g6), ask me on [instagram](https://instagram.com/draverindustries) or by mail at `draver.industries@proton.me`
