const { log } = require('node:console');
const Detector = require('./Detector');
const { logMessage } = require('./logger');
const { writeFileSync, existsSync } = require('node:fs');

const engine = new Detector();

engine.on('ready', async() => {
    if (process.argv.includes('--recalc')) {
        engine.recalc()
        return  
    }
    
    const createBrains = (amount) => {
        const map = []
        
        for (let i = 0; i < amount; i++) {
            map.push(engine.data.randomImportance)
        }
        
        return map
    }
    
    const brains = (() => {
        if (existsSync('./brains.json')) return require('./brains.json')
            return createBrains(49).concat([
            engine.data.importance
        ])
    })();
    
    const generation = async(brainList, gen = 0) => {
        const start = Date.now()
        const mapper = (brain, index) => {
            return new Promise((resolve) => {
                logMessage('info', `Testing brain ${index}...`)
                engine.test(8, brain).then((result) => {
                    logMessage('info', `Brain ${index} has a score of ${result}`)
                    return resolve([index, result])
                })
                
            })
        }
        
        const results = []
        for (const index in brainList) {
            const res = await mapper(brainList[index], index)
            results.push(res)
        }  
        const sorted = results.sort((a, b) => a[1] - b[1])
        
        const bestBrains = sorted.slice(0, 13)
        
        logMessage('info', `Generation ${gen} took ${Date.now() - start}ms`)
        logMessage('info', `Best brain for generation ${gen}: ${bestBrains[0][1]}`)
        logMessage('info', `Worst brain for generation ${gen}: ${bestBrains[bestBrains.length - 1][1]}`)
        logMessage('info', `Going to next generation... ${gen + 1}`)
        
        const newGen = bestBrains.map((brain) => {
            const origin = brainList[brain[0]]
            const mutated = engine.data.mutate(origin)
            
            return [origin, mutated]
        }).flat().concat(createBrains(24));
        writeFileSync('./brains.json', JSON.stringify(newGen))
        
        generation(newGen, gen + 1)    
        
    }
    
    logMessage('info', 'Engine is ready, starting generation...')
    generation(brains)
})