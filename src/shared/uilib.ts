import type { App } from 'vue';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Dialog from 'primevue/dialog';
import Tabs from 'primevue/tabs';
import TabList from 'primevue/tablist';
import Tab from 'primevue/tab';
import Splitter from 'primevue/splitter';
import SplitterPanel from 'primevue/splitterpanel';
import Drawer from 'primevue/drawer';
import Menu from 'primevue/menu';
import ContextMenu from 'primevue/contextmenu';
import { Form, FormField } from '@primevue/forms';


// ...其他需要的 PrimeVue 组件...

export default {
    install(app: App) {
        app.component('Button', Button);
        app.component('InputText', InputText);
        app.component('Dialog', Dialog);
        app.component('Tabs', Tabs);
        app.component('TabList', TabList);
        app.component('Tab', Tab);
        app.component('Splitter', Splitter);
        app.component('SplitterPanel', SplitterPanel);
        app.component('Drawer', Drawer);
        app.component('Menu', Menu);
        app.component('ContextMenu', ContextMenu);
        app.component('Form', Form);
        app.component('FormField', FormField);
    }
};
