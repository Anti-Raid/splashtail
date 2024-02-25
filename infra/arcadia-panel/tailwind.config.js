const tones = Array.from({ length: 10 }).reduce((obj, val, index) => {
	const tone = index == 0 ? '50' : String(index * 100);
	const tones = Array.from({ length: 10 }).reduce((_obj, _val, _index) => {
		const _tone = String(_index) + (_index != 0 ? '0' : '');
		_obj[tone + '/' + _tone] = `var(--${tone}-${_tone})`;
		return _obj;
	}, {});
	obj = {
		...tones,
		...obj
	};
	return obj;
}, {});

/** @type {import('tailwindcss').Config}*/
const config = {
	mode: 'jit',
	important: true,
	content: ['./src/**/*.{html,js,svelte,ts}'],
	darkMode: 'none',
	theme: {
		extend: {
			hueRotate: {
				0: 'var(--500)'
			},
			colors: {
				// ShadCN
				border: 'hsl(var(--border))',
				input: 'hsl(var(--input))',
				ring: 'hsl(var(--ring))',
				background: 'hsl(var(--background))',
				foreground: 'hsl(var(--foreground))',
				primary: {
					DEFAULT: 'hsl(var(--primary))',
					foreground: 'hsl(var(--primary-foreground))'
				},
				secondary: {
					DEFAULT: 'hsl(var(--secondary))',
					foreground: 'hsl(var(--secondary-foreground))'
				},
				destructive: {
					DEFAULT: 'hsl(var(--destructive))',
					foreground: 'hsl(var(--destructive-foreground))'
				},
				muted: {
					DEFAULT: 'hsl(var(--muted))',
					foreground: 'hsl(var(--muted-foreground))'
				},
				accent: {
					DEFAULT: 'hsl(var(--accent))',
					foreground: 'hsl(var(--accent-foreground))'
				},
				popover: {
					DEFAULT: 'hsl(var(--popover))',
					foreground: 'hsl(var(--popover-foreground))'
				},
				card: {
					DEFAULT: 'hsl(var(--card))',
					foreground: 'hsl(var(--card-foreground))'
				},

				// Reed
				dark: '#16151d',
				waohhh: '#000000',
				cards: '#191919',
				carrd: '#000000',
				violet_samp: 'rgba(139, 92, 246, .50)',
				blue_samp: 'rgba(59, 130, 246, .50)',
				emerald_samp: 'rgba(16, 185, 129, .50)',
				rose_samp: 'rgba(239, 68, 68, .50)',
				summer_samp: 'rgb(250, 95, 100)',
				themable_samp: 'rgba(245, 158, 11, .50)',
				modes: {
					dark: '#16151d',
					light: ''
				},
				themable: {
					...tones,
					900: 'var(--900)',
					800: 'var(--800)',
					700: 'var(--700)',
					600: 'var(--600)',
					500: 'var(--500)',
					400: 'var(--400)',
					300: 'var(--300)',
					200: 'var(--200)',
					100: 'var(--100)',
					50: 'var(--50)'
				}
			},

			// ShadCN
			borderRadius: {
				lg: `var(--radius)`,
				md: `calc(var(--radius) - 2px)`,
				sm: 'calc(var(--radius) - 4px)'
			},
			keyframes: {
				'accordion-down': {
					from: { height: '0' },
					to: { height: 'var(--radix-accordion-content-height)' }
				},
				'accordion-up': {
					from: { height: 'var(--radix-accordion-content-height)' },
					to: { height: '0' }
				}
			},
			animation: {
				'accordion-down': 'accordion-down 0.2s ease-out',
				'accordion-up': 'accordion-up 0.2s ease-out'
			}
		}
	},
	variants: {
		extend: {}
	},
	plugins: [require('@tailwindcss/typography'), require('tailwindcss-animate')]
};

module.exports = config;
