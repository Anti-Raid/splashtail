const log = (...args) => {
    console[args[0]](
        `%c[${Date.now()}]%c[${args[1]}]%c`,
        'color:red;font-weight:bold;',
        'color:purple;font-weight:bold;',
        '',
        ...args.slice(2)
    )
}

// Custom logger
const debug = (...args) => log('debug', ...args)
const info = (...args) => log('info', ...args)
const warn = (...args) => log('warn', ...args)
const error = (...args) => log('error', ...args)

const exportedObject = {
    debug,
    info,
    warn,
    error
}

export default exportedObject;
