/*!
 * Color mode toggler for Bootstrap's docs (https://getbootstrap.com/)
 * Copyright 2011-2023 The Bootstrap Authors
 * Adapted to typescript for Rustime
 * Licensed under the Creative Commons Attribution 3.0 Unported License.
 */
(() => {
	'use strict'

	const storedTheme: string | null = localStorage.getItem('theme');

	const getPreferredTheme = (): string => {
		if (storedTheme) return storedTheme;
		return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
	}

	const setTheme = (theme: string): void => {
		if (theme === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
			document.documentElement.setAttribute('data-bs-theme', 'dark');
		} else {
			document.documentElement.setAttribute('data-bs-theme', theme);
		}
	}

	setTheme(getPreferredTheme());

	const showActiveTheme = (theme: string, focus: boolean = false): void => {
		const themeSwitcher: Element | null = document.querySelector('#bd-theme');

		if (!themeSwitcher) return;

		const themeSwitcherText: Element | null = document.querySelector('#bd-theme-text');
		const activeThemeIcon: Element | null = document.querySelector('.theme-icon-active use');
		const btnToActive: Element | null = document.querySelector(`[data-bs-theme-value="${theme}"]`);
		const svgOfActiveBtn: string | null | undefined = btnToActive?.querySelector('svg use')?.getAttribute('href');

		if (!themeSwitcherText || !activeThemeIcon || !btnToActive || !svgOfActiveBtn) return;

		for (const element of document.querySelectorAll('[data-bs-theme-value]')) {
			element.classList.remove('active');
			element.setAttribute('aria-pressed', 'false');
		}

		btnToActive.classList.add('active');
		btnToActive.setAttribute('aria-pressed', 'true');
		activeThemeIcon.setAttribute('href', svgOfActiveBtn);
		// @ts-ignore
		const themeSwitcherLabel: string = `${themeSwitcherText.textContent} (${btnToActive.dataset.bsThemeValue})`;
		themeSwitcher.setAttribute('aria-label', themeSwitcherLabel);

		// @ts-ignore
		if (focus) themeSwitcher.focus();
	}

	window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (): void => {
		// @ts-ignore
		if (storedTheme !== 'light' || storedTheme !== 'dark') {
			setTheme(getPreferredTheme());
		}
	});

	window.addEventListener('DOMContentLoaded', (): void => {
		showActiveTheme(getPreferredTheme());

		for (const toggle of document.querySelectorAll('[data-bs-theme-value]')) {
			toggle.addEventListener('click', (): void => {
				const theme: string = toggle.getAttribute('data-bs-theme-value') || '';
				localStorage.setItem('theme', theme);
				setTheme(theme);
				showActiveTheme(theme, true);
			});
		}
	});
})();