import {Injectable} from '@angular/core';
import {ActivatedRouteSnapshot, Resolve, RouterStateSnapshot} from '@angular/router';
import {Observable} from 'rxjs';
import {UserViewModelService} from './user-view-model.service';
import {User} from '../models';

@Injectable({
  providedIn: 'root'
})
export class UserResolveService implements Resolve<User> {

  constructor(
    private viewModel: UserViewModelService,
  ) {
  }

  resolve(route: ActivatedRouteSnapshot, state: RouterStateSnapshot): Observable<User> | Promise<User> | any {
    return this.viewModel.user().toPromise();
  }
}
