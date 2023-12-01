import postgres from 'postgres'

const sql = postgres({ 
    database: 'antiraid',
    transform: postgres.fromCamel,
    password: 'password',
 })

export default sql