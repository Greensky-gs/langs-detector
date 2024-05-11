const { logMessage } = require('./logger');
const sequelize = require('sequelize')
const axios = require('axios')
const { parse: htmlParse } = require('node-html-parser');
const { action } = require('./action');
const { readdirSync, existsSync, rmSync, readFileSync, rm, writeFileSync } = require('node:fs');
const { log } = require('node:console');
const EventEmitter = require('node:events');

class Data {
    alphabet = "abcdefghijklmnopqrstuvwxyzß"
    accents = "áéíóúàèìòùâêîôûäëïöüãẽĩõũçñåøčšžđý"
    punctuation = "-'¿¡"
    groups = ["ss", "nn", 'tt', 'rr', 'll', 'mm', 'cc', 'ff', 'pp', 'gg', 'zz', 'bb', 'cc', 'ch', "sh", "qu", 'sch', 'ei', 'kr', "sn", "bn", "ou", 'rn', 'nm', 'ls']
    terminations = ['ing', 'ant', "ght", 'gth', 'ons', 'ont', 'na', 'no', 'der', 'den', 'das', 'die', 'ein', 'eine', 'einen', 'einem', 'einer', 'eines', 'ion', 'tion', 'ez', 'ent']
    aloneLetters = "aàiuå"
    markedPunctuation = "?!:"
    allowed = this.alphabet + this.accents + this.punctuation + this.aloneLetters + this.markedPunctuation

    importance = {
        ç: 800,
        é: 3,
        e: 2,
        u: 1.5,
        h: 1.5,
        z: 1.5,
        w: 2,
        x: 2,
        averageWordSize: 0.2,
        averageCapitalsSize: 0.2,
        "'": 0.8,
        na: 0.8,
        no: 0.8,
        "ß": 300,
        "č": 3,
        "š": 3,
        "ž": 3,
        "đ": 3,
        "¡": 800,
        "¿": 800,
        "á": 3,
        "-": 0.5,
        l: 0.7,
        m: 0.7,
        n: 0.7,
        o: 0.7,
        p: 1.2,
        q: 1.2,
        r: 1.1,
        s: 1.1,
        t: 1.1,
        u: 1.2,
        v: 1.2,
        y: 2,
        z: 2,
    }

    constructor() {
        this.terminations.concat(this.groups).forEach(x => this.importance[x] = 1.5)
    }

    get randomImportance() {
        return Object.fromEntries(Object.keys(this.importance).map(x => [x, Math.floor(Math.random() * 100)]))
    }
    mutate(template) {
        const newImportance = {};
        for (const key in template) {
            const originalValue = template[key];
            const minValue = originalValue * 0.7;
            const maxValue = originalValue * 1.3;
            const randomValue = Math.floor(Math.random() * (maxValue - minValue + 1) + minValue * 10) / 10;
            newImportance[key] = randomValue;
        }

        return newImportance
    }
}

const getStringOpt = (optName, deft) => {
    const opt = process.argv.find(x => x === optName)
    if (!opt) return deft

    const index = process.argv.indexOf(opt)
    const value = process.argv[index + 1]
    
    if (!value || value.startsWith('--')) return deft
    return value
}

class Detector extends EventEmitter {
    /**
     * @type {sequelize.Sequelize}
     */
    database
    /**
     * @type {sequelize.Model}
     */
    langs

    ready = false

    /**
     * @type {{ name: string; ratios: Record<string, number>; content: string; }[]}
     */
    vectors = []
    data = new Data()

    counts = {}

    constructor() {
        super({
            captureRejections: true
        })
        this.init()
    }

    /**
     * @param {string} input 
     */
    calculateRatio(input) {
        const cleanChars = input.toLowerCase().split("").filter(char => (this.data.allowed + ' ').includes(char)).join("")
        
        const result = {}
        const pure = {}

        for (const char of this.data.allowed.split('').filter(x => !this.data.markedPunctuation.includes(x))) {
            const regex = new RegExp(char, "gi")
            result[char] = cleanChars.match(regex)?.length ?? 0
        }
        for (const double of this.data.groups) {
            const regex = new RegExp(double, "gi")
            result[double] = cleanChars.match(regex)?.length ?? 0
        }
        for (const termination of this.data.terminations) {
            const regex = new RegExp(`(${termination})(?=( |\\.|,))`, "gi")
            result[termination] = cleanChars.match(regex)?.length ?? 0
        }

        const averageWordSize = input.split(" ").map(x => x.length).reduce((a, b) => a + b, 0) / input.split(" ").length
        pure.words = averageWordSize

        const averageCapitalsSize = input.split(" ").map(x => x.split("").filter(x => x === x.toUpperCase()).length).reduce((a, b) => a + b, 0) / input.split(" ").length
        pure.capitals = averageCapitalsSize

        const aloneLettersRatio = input.split(/ +/).filter(x => x.length === 1).length / input.split(/ +/g).length
        pure.aloneLetters = aloneLettersRatio

        for (const punctuation of this.data.markedPunctuation) {
            const far = new RegExp(`(?<= )(\\${punctuation})`, 'gi')
            const close = new RegExp(`(?! )(\\${punctuation})`, 'gi')

            result[`punct-${punctuation}-far`] = cleanChars.match(far)?.length ?? 0
            result[`punct-${punctuation}-close`] = cleanChars.match(close)?.length ?? 0
        }
        for (const alone of this.data.aloneLetters) {
            const regex = new RegExp(`((?<=(\\.| |,))${alone}(?=(\\.| |,|$)))`, 'gi')

            result[`alone-${alone}`] = cleanChars.match(regex)?.length ?? 0
        }
    
        return Object.fromEntries(Object.entries(result).map(([k, v]) => [k, v / cleanChars.length]).concat(Object.entries(pure)))
    }

    /**
     * @param {string} input
     * @param {number} k
     * @returns {Promise<string>}
     * @param {{ importance?: Record<string, number> }} data
     */
    async detect(input, k, data = {}) {
        const importance = data?.importance ?? this.data.importance

        return new Promise((resolve, reject) => {
            const ratios = this.calculateRatio(input)

            const mapped = this.vectors.map(lang => {
                const distance = Math.sqrt(Object.entries(ratios).map(([k, v]) => (lang.ratios[k] * (importance[k] ?? 1) - v * (importance[k] ?? 1)) ** 2).reduce((a, b) => a + b, 0))
                return { distance, lang }
            })

            const sorted = mapped.sort((a, b) => a.distance - b.distance);

            const best = sorted.slice(0, k)
        
            const counts = {}
            for (const { lang } of best) {
                counts[lang.name] = (counts[lang.name] ?? 0) + 1
            }

            const mostCommon = Object.entries(counts).sort((a, b) => b[1] - a[1])[0][0]
            return resolve(mostCommon)
        })
    }

    async append(input, lang) {
        const ratios = this.calculateRatio(input)

        await langs.create({
            name: lang,
            ratios,
            content: input
        })
        return true
    }
    async test(k, imp) {
        return new Promise(async(resolve) => {
            const tests = [
                {
                    input: "Hello, world! This is a test sentence in English.",
                    expected: "en"
                },
                {
                    input: "Bonjour tout le monde! Ceci est une phrase de test en français.",
                    expected: "fr"
                },
                {
                    input: "Guten Tag, Welt! Dies ist ein Test Satz auf Deutsch.",
                    expected: "de"
                },
                {
                    input: "I love programming and solving complex problems.",
                    expected: "en"
                },
                {
                    input: "J'adore programmer et résoudre des problèmes complexes.",
                    expected: "fr"
                },
                {
                    input: "Ich liebe Programmieren und das Lösen komplexer Probleme.",
                    expected: "de"
                },
                {
                    input: "The quick brown fox jumps over the lazy dog.",
                    expected: "en"
                },
                {
                    input: "Le renard brun rapide saute par-dessus le chien paresseux.",
                    expected: "fr"
                },
                {
                    input: "Der schnelle braune Fuchs springt über den faulen Hund.",
                    expected: "de"
                },
                {
                    input: "I enjoy reading books and learning new things.",
                    expected: "en"
                },
                {
                    input: "J'aime lire des livres et apprendre de nouvelles choses.",
                    expected: "fr"
                },
                {
                    input: "Ich genieße es, Bücher zu lesen und neue Dinge zu lernen.",
                    expected: "de"
                }
            ];
            
            if (!k || isNaN(k) || k < 1) {
                const log = []
    
                for (let kr = 5; kr < 100; kr++) {
                    const start = Date.now()
                    let success = 0
        
                    for (const test of tests) {
                        const result = await this.detect(test.input, kr, {
                            importance: imp
                        });
    
    
                        if (result === test.expected) success++
                    }
                    const end = Date.now()
                    const computed = end - start
    
                    log.push({
                        k: kr,
                        success,
                        tests: tests.length,
                        rate: success / tests.length,
                        time: computed
                    })
    
                    console.log(`Success rate for k=${kr}: ${success}/${tests.length}`)
                }
    
                writeFileSync('./log.json', JSON.stringify(log, null, 4))
                resolve(true)
            } else {
                let success = 0
        
                const promises = tests.map((test) => new Promise(async(resolve) => {
                    const result = await this.detect(test.input, k, {
                        importance: imp
                    });

                    if (result === test.expected) success++
                    resolve()
                }))
                await Promise.all(promises)
    
                resolve(success/tests.length)
            }

        })
    }

    async recalc() {
        return new Promise((resolve) => {
            const start = Date.now()
            logMessage('info', "Recalculating the ratios for all the vectors...")
        
            const size = this.vectors.length
            let done = 0

            const mapper = (lang) => {
                const ratios = this.calculateRatio(lang.content)
                lang.ratios = ratios

                this.langs.update({ ratios }, { where: { id: lang.id } }).then(() => {
                    done++
                    if (done === size) {
                        logMessage('success', `All the ratios have been recalculated in \x1b[38;5;220m${(Date.now() - start).toLocaleString('fr')}\x1b[0mms`)

                        resolve(true)
                    } else {
                        logMessage('info', `Recalculated ${done.toLocaleString('fr')}/${size.toLocaleString('fr')} vectors`)
                    }
                }).catch(err => {
                    logMessage('error', "Unable to update the ratios for the vectors", err)
                })
            }

            this.vectors.forEach(mapper.bind(this))
        })
    }

    async startScrapper(method = 'wikipedia') {
        let scrapped = 0;

        const url = () => {
            const list = [
                ["https://fr.wikipedia.org/wiki/Sp%C3%A9cial:Page_au_hasard", "fr", () => this.counts.french, () => this.counts.french++],
                ["https://en.wikipedia.org/wiki/Special:Random", "en", () => this.counts.english, () => this.counts.english++],
                ["https://es.wikipedia.org/wiki/Especial:Aleatoria", "es", () => this.counts.spanish, () => this.counts.spanish++],
                ["https://it.wikipedia.org/wiki/Speciale:PaginaCasuale", "it", () => this.counts.italiano, () => this.counts.italiano++],
                ["https://de.wikipedia.org/wiki/Spezial:Zuf%C3%A4llige_Seite", "de", () => this.counts.deutch, () => this.counts.deutch++],
                ["https://hr.wikipedia.org/wiki/Posebno:Slu%C4%8Dajna_stranica", "hr", () => this.counts.hr, () => this.counts.hr++],
                ["https://cs.wikipedia.org/wiki/Speci%C3%A1ln%C3%AD:N%C3%A1hodn%C3%A1_str%C3%A1nka", "cs", () => this.counts.cs, () => this.counts.cs++],
                ["https://da.wikipedia.org/wiki/Speciel:Tilf%C3%A6ldig_side", "da", () => this.counts.da, () => this.counts.da++],
                ["https://nl.wikipedia.org/wiki/Speciaal:Willekeurig", "nl", () => this.counts.nl, () => this.counts.nl++],
                ["https://no.wikipedia.org/wiki/Spesial:Tilfeldig_side", "no", () => this.counts.no, () => this.counts.no++]
            ]
            
            const leastDataLang = Math.min(Object.values(this.counts));
            const leastDataLangIndex = Object.values(this.counts).indexOf(leastDataLang);
            
            return list[leastDataLangIndex];
        }

        const wait = (ms) => new Promise(resolve => setTimeout(resolve, ms))
        const wikipediaScrap = async() => {
            const [target, lang, count, increaser] = url()
            logMessage('info', `Scraping a new random wikipedia page in \x1b[38;5;220m${lang}\x1b[0m`);

            const recurse = async() => {
                const time = Math.floor(Math.random() * 2000) + 1000;
                logMessage('info', `Retrying in ${time}ms`)

                await wait(time)
                wikipediaScrap()
            }


            const result = await axios.get(target).catch(err => {
                logMessage('error', "Unable to fetch the random wikipedia page", err)
            })
            if (!result || result.status !== 200) {
                logMessage('unexpected', "The wikipedia page returned an unexpected status code")
                recurse()
                return
            }

            const html = result.data
            logMessage('info', `Interpreting ${result.request?.res?.responseUrl}`)

            const parsed = htmlParse(html)

            const texts = parsed.querySelectorAll(".mw-body-content .mw-content-ltr p").map((p) => {
                return p.textContent
            })

            if (texts.length <= 4) {
                logMessage('unexpected', "Not enough text to analyze")
                recurse()
                return
            }

            logMessage("success", "Successfully fetched the random wikipedia page!")
            scrapped++
            this.append(texts.join("\n"), lang)

            increaser()

            recurse()
        }
        const randomLang = (available) => available[Math.floor(Math.random() * available.length)]
        const gutenbergScrap = async(available = ["fr", "en", "es", "it", "de", "hr"], lang = getStringOpt('--lang', randomLang(available))) => {
            const url = `https://www.gutenberg.org/browse/languages/${lang}`

            logMessage("info", `Getting all books in the gutenberg page for \x1b[38;5;220m${lang}\x1b[0m`)
            const result = await axios.get(url).catch(err => {
                logMessage('error', "Unable to fetch the gutenberg page", err)
                wait(2000).then(() => gutenbergScrap(available, lang))
            })
            if (!result || result.status !== 200) {
                logMessage('unexpected', "The gutenberg page returned an unexpected status code")
                wait(2000).then(() => gutenbergScrap(available, lang))
                return
            }

            const content = htmlParse(result.data)
            const books = content.querySelectorAll(".pgdbbylanguage ul li a").map(x => x.rawAttrs.split("/")[2].replace('"', ''))

            if (books.length === 0) {
                logMessage('unexpected', "No books found for the language")

                available = available.filter(x => x !== lang)

                wait(2000).then(() => gutenbergScrap(available, randomLang(available)))
                return
            }
            const urls = books.map(x => `https://www.gutenberg.org/cache/epub/${x}/pg${x}.txt`).map(x => [x, Math.random()]).sort((a, b) => a[1] - b[1]).map(x => x[0])
            logMessage('info', `Found ${books.length} books to scrap`)

            const length = urls.length

            const recurse = async(book, tries = 0) => {
                const done = length - urls.length

                if (tries === 3) {
                    logMessage('unexpected', "Unable to fetch the book after 3 tries, skipping")
                    recurse(urls.shift())
                    return
                }

                logMessage('info', `Fetching the book ${book}`)
                const result = await axios.get(book).catch(err => {
                    logMessage('error', "Unable to fetch the book", err)
                    return
                })
                if (!result || result.status !== 200) {
                    logMessage('unexpected', "The book returned an unexpected status code")
                    wait(2000).then(() => recurse(book, tries + 1))
                    return
                }

                const txt = result.data
                const text = txt.split(/\*\*\*.+\*\*\*/)[1]

                if (!text) {
                    logMessage('unexpected', "The book does not contain any text")
                    recurse(urls.shift())
                    return
                }
                const paragraphs = text.split(/(\n){1,}/g).filter(x => x.length > 40)

                paragraphs.forEach(x => this.append(x, lang))
                scrapped++
                logMessage('success', `Successfully fetched the book ${book}, with ${paragraphs.length} paragraphs (\x1b[38;5;220m${done}\x1b[0m/${length})`)

                wait(2000).then(() => recurse(urls.shift()))
            }

            recurse(urls.shift())
        }

        switch (method) {
            case 'wikipedia':
                wikipediaScrap()
                break;
            case 'gutenberg':
                gutenbergScrap()
                break;
            default:
                throw new Error("The site specified is not supported")
        }

        setInterval(() => {
            logMessage('info', `Scrapped \x1b[38;5;220m${scrapped.toLocaleString('fr')}\x1b[0m pages a minute`)
            scrapped = 0
        }, 60000)
    }

    async train() {
        if (!existsSync("./train")) {
            throw new Error("The train directory does not exist")
        }

        readdirSync("./train").filter(x => !x.includes('.')).forEach(langCode => {
            logMessage('info', `Training the language \x1b[38;5;220m${langCode}\x1b[0m`)

            const files = readdirSync(`./train/${langCode}`)
            logMessage('info', `Found ${files.length} files to train`)

            files.forEach(file => {
                const content = readFileSync(`./train/${langCode}/${file}`).toString()
                this.append(content, langCode)

                rmSync(`./train/${langCode}/${file}`)
            })
        })
    }

    async init() {
        logMessage('info', 'Starting the application...');

        this.database = new sequelize.Sequelize({
            dialect: "mysql",
            username: "root",
            database: 'langs',
            password: '',
            host: 'localhost',
            logging: false
        })

        await this.database.authenticate()
        await this.database.sync({ alter: true }).then(() => {
            logMessage('info', 'Database connection established successfully!');
        }).catch((err) => {
            logMessage('DB', 'Unable to connect to the database:', err);
        })

        logMessage('info', "Defining the database schema...")

        this.langs = this.database.define("langs", {
            name: {
                type: sequelize.STRING,
                allowNull: false
            },
            ratios: {
                type: sequelize.JSON,
                allowNull: false
            },
            content: {
                type: sequelize.TEXT,
                allowNull: false
            }
        })

        await this.langs.sync({  }).then(() => {
            logMessage('info', 'Database schema created successfully!');
        }).catch((err) => {
            logMessage('DB', 'Unable to create the database schema:', err);
        })
        
        this.vectors = (await this.langs.findAll()).map(x => ({ ...x.dataValues, ratios: JSON.parse(x.dataValues.ratios) }))

        logMessage('info', `Loaded \x1b[38;5;220m${this.vectors.length.toLocaleString('fr')}\x1b[0m vectors`)
        if (this.vectors[0]) logMessage('info', `These are ${Object.keys(this.vectors[0].ratios).length} dimensions vectors`)

        for (const [key, code] of [["fr", "french"], ["es", "spanish"], ["it", "italiano"], ["de", "deutch"], ["hr", "hr"], ["cs", "cs"], ["da", "da"], ["nl", "nl"], ["no", "no"], ["en", "english"]]) {
            this.counts[key] = this.vectors.filter(x => x.name === code).length
        }

        this.emit('ready')
        this.ready = true
    }
}

module.exports = Detector

if (process.argv.includes("--no-error")) {
    process.on('uncaughtException', () => {})
    process.on('unhandledRejection', () => {})
    process.on('uncaughtExceptionMonitor', () => {})
}