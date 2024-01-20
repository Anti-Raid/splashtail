export const logoutUser = () => {
    localStorage.removeItem('wistala')
    localStorage.clear()
}