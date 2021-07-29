import {NgModule} from '@angular/core';
import {RouterModule, Routes} from '@angular/router';
import {ControllerComponent} from './components/controller/controller.component';
import {ControllersComponent} from './components/controllers/controllers.component';

const routes: Routes = [
  {path: 'controllers', component: ControllersComponent},
  {path: 'controller/:id', component: ControllerComponent}
];

@NgModule({
  imports: [RouterModule.forChild(routes)],
  exports: [RouterModule]
})
export class AdminRoutingModule {
}
