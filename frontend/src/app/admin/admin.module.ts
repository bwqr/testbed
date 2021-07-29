import { NgModule } from '@angular/core';
import { CommonModule } from '@angular/common';

import { AdminRoutingModule } from './admin-routing.module';
import { ControllerComponent } from './components/controller/controller.component';
import { ControllersComponent } from './components/controllers/controllers.component';
import { CoreModule } from '../core/core.module';
import { SharedModule } from '../shared/shared.module';


@NgModule({
    declarations: [
        ControllerComponent,
        ControllersComponent,
    ],
    imports: [
        CommonModule,
        AdminRoutingModule,
        CoreModule,
        SharedModule,
    ]
})
export class AdminModule {
}
