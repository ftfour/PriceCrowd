import { createRouter, createWebHistory } from 'vue-router';

const Home = () => import('../views/Home.vue');
const About = () => import('../views/About.vue');
const Products = () => import('../views/Products.vue');
const ProductDetail = () => import('../views/ProductDetail.vue');
const ProductForm = () => import('../views/ProductForm.vue');
const Stores = () => import('../views/Stores.vue');
const StoreDetail = () => import('../views/StoreDetail.vue');
const StoreForm = () => import('../views/StoreForm.vue');
const StoreActivity = () => import('../views/StoreActivity.vue');
const Cart = () => import('../views/Cart.vue');
const Categories = () => import('../views/Categories.vue');
const CategoryForm = () => import('../views/CategoryForm.vue');

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', name: 'home', component: Home },
    { path: '/about', name: 'about', component: About },
    { path: '/products', name: 'products', component: Products },
    { path: '/products/new', name: 'product-new', component: ProductForm },
    { path: '/products/:id', name: 'product-detail', component: ProductDetail },
    { path: '/products/:id/edit', name: 'product-edit', component: ProductForm },
    { path: '/stores', name: 'stores', component: Stores },
    { path: '/stores/new', name: 'store-new', component: StoreForm },
    { path: '/stores/:id', name: 'store-detail', component: StoreDetail },
    { path: '/stores/:id/edit', name: 'store-edit', component: StoreForm },
    { path: '/stores/:id/activity', name: 'store-activity', component: StoreActivity },
    { path: '/cart', name: 'cart', component: Cart },
    { path: '/categories', name: 'categories', component: Categories },
    { path: '/categories/new', name: 'category-new', component: CategoryForm },
    { path: '/categories/:id/edit', name: 'category-edit', component: CategoryForm },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
  scrollBehavior() {
    return { top: 0 };
  },
});

export default router;
