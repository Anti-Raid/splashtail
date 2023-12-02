// @ts-nocheck

// Logging helper


const log = (...args: any[]) => {
    console[args[0]](
      `%c[${Date.now()}]%c[${args[1]}]%c`,
      'color:red;font-weight:bold;',
      'color:purple;font-weight:bold;',
      '',
      ...args.slice(2)
    );
  };
  
  export const info = (...args: any[]) => {
    log('info', ...args);
  };
  
  export const debug = (...args: any[]) => {
    log('debug', ...args);
  };
  
  export const warn = (...args: any[]) => {
    log('warn', ...args);
  };
  
  export const error = (...args: any[]) => {
    log('error', ...args);
  };