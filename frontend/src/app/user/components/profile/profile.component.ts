import {Component, OnInit} from '@angular/core';
import {MainComponent} from '../../../shared/components/main/main.component';
import {User} from '../../models';
import {UserViewModelService} from '../../services/user-view-model.service';

@Component({
  selector: 'app-profile',
  templateUrl: './profile.component.html',
  styleUrls: ['./profile.component.scss']
})
export class ProfileComponent extends MainComponent implements OnInit {

  user: User;

  get isPageReady(): boolean {
    return !!this.user;
  }

  constructor(
    private viewModel: UserViewModelService,
  ) {
    super();
  }

  ngOnInit(): void {
    this.subs.add(
      this.viewModel.profile().subscribe(user => this.user = user)
    );
  }

}
