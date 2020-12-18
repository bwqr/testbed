import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';

import {UserRoutingModule} from './user-routing.module';
import {ProfileComponent} from './components/profile/profile.component';
import {CoreModule} from '../core/core.module';
import {SharedModule} from '../shared/shared.module';
import { SettingsComponent } from './components/settings/settings.component';


@NgModule({
  declarations: [ProfileComponent, SettingsComponent],
  imports: [
    CommonModule,
    UserRoutingModule,
    CoreModule,
    SharedModule
  ]
})
export class UserModule {
}
