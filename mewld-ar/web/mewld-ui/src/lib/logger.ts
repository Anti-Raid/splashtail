

// Logging helper
const log = (...args) => {
    console[args[0]](
      `%c[${Date.now()}]%c[${args[1]}]%c`,
      'color:red;font-weight:bold;',
      'color:purple;font-weight:bold;',
      '',
      ...args.slice(2)
    );
  };
  
  export const info = (...args) => {
    log('info', ...args);
  };
  
  export const debug = (...args) => {
    log('debug', ...args);
  };
  
  export const warn = (...args) => {
    log('warn', ...args);
  };
  
  export const error = (...args) => {
    log('error', ...args);
  };