import { mount } from 'svelte';
import '../styles/tokens.css';
import Settings from './Settings.svelte';

mount(Settings, { target: document.getElementById('app')! });
