import {Component, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {User} from '../../models';
import {UserViewModelService} from '../../services/user-view-model.service';
import {FormBuilder, FormGroup, Validators} from '@angular/forms';
import {finalize} from 'rxjs/operators';

@Component({
  selector: 'app-profile',
  templateUrl: './profile.component.html',
  styleUrls: ['./profile.component.scss']
})
export class ProfileComponent extends MainComponent implements OnInit {

  user: User;

  formGroup: FormGroup;

  get isPageReady(): boolean {
    return !!this.user;
  }

  constructor(
    private viewModel: UserViewModelService,
    private formBuilder: FormBuilder,
  ) {
    super();

    this.formGroup = formBuilder.group({
      firstName: formBuilder.control('', [Validators.required, Validators.maxLength(122)]),
      lastName: formBuilder.control('', [Validators.required, Validators.maxLength(122)])
    });
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.user().subscribe(user => {
        this.user = user;

        this.formGroup.patchValue({firstName: user.firstName, lastName: user.lastName});
      })
    );
  }

  updateProfile(value: any): void {
    this.enterProcessingState();

    this.subs.add(
      this.viewModel.updateProfile(value.firstName, value.lastName).pipe(
        finalize(() => this.leaveProcessingState())
      )
        .subscribe(_ => {
          this.user.firstName = value.firstName;
          this.user.lastName = value.lastName;
        })
    );
  }
}
