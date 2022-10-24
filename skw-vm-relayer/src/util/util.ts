import pino from 'pino';

export const logger = pino({
  transport: {
    options: {
      colorize: true,
      ignore: 'hostname',
      singleLine: true,
      translateTime: 'yyyy-mm-dd HH:MM:ss'
    },
    target: 'pino-pretty'
  }
});

export const numberPadding = (n: number, pad: number): string => {
  return String(n).padStart(pad, '0');
};