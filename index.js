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
        if (existsSync('./brains.json')) return require('./brains.json').brains
            return createBrains(49).concat([
            engine.data.importance
        ])
    })();
    const currentGen = (() => {
        if (existsSync('./brains.json')) return require('./brains.json').gen
        return 0
    })();
    
    const generation = async(brainList, gen = 0, k = 8) => {
        const start = Date.now()
        const mapper = (brain, index) => {
            return new Promise((resolve) => {
                logMessage('info', `Testing brain ${index}...`)
                engine.test(k, brain).then((result) => {
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
        const sorted = results.sort((a, b) => b[1] - a[1])
        
        const bestBrains = sorted.slice(0, 13)
        
        logMessage('info', `Generation ${gen} took ${Math.floor((Date.now() - start) / 1000).toLocaleString('fr')}s`)
        logMessage('info', `Best brain for generation ${gen}: ${sorted[0][1]}`)
        logMessage('info', `Worst brain for generation ${gen}: ${sorted[sorted.length - 1][1]}`)
        logMessage('info', `Going to next generation... ${gen + 1}`)
        
        const newGen = bestBrains.map((brain) => {
            const origin = brainList[brain[0]]
            const mutated = engine.data.mutate(origin)
            
            return [origin, mutated]
        }).flat().concat(createBrains(24));
        writeFileSync('./brains.json', JSON.stringify({
            brains: newGen,
            gen: gen + 1
        }))
        
        if (bestBrains[0][1] <= 0.5 && gen > 8) k = 12
        generation(newGen, gen + 1, k)
        
    }
    
    logMessage('info', 'Engine is ready, starting generation...')
    generation(brains, currentGen)
})