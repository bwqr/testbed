import {NgModule} from '@angular/core';
import {CommonModule} from '@angular/common';
import {MainComponent} from './components/main/main.component';
import {ReactiveFormsModule} from '@angular/forms';
import {HttpClientModule} from '@angular/common/http';
import {ErrorMessageComponent} from './components/error-message/error-message.component';
import {FormValidityDirective} from './directives/form-validity.directive';
import {RouterModule} from '@angular/router';


@NgModule({
  declarations: [
    MainComponent,
    ErrorMessageComponent,
    FormValidityDirective,
  ],
  imports: [
    CommonModule,
  ],
  exports: [
    ReactiveFormsModule,
    HttpClientModule,
    ErrorMessageComponent,
    FormValidityDirective,
    RouterModule
  ]
})
export class SharedModule {
}
